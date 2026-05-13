use chrono::Utc;
use opengithub_api::domain::{
    identity::upsert_user_by_email,
    permissions::RepositoryRole,
    repositories::{
        create_organization, create_repository, get_repository_by_owner_name, insert_commit,
        list_repositories_for_user, repository_permission_for_user, upsert_git_ref, CreateCommit,
        CreateOrganization, CreateRepository, RepositoryOwner, RepositoryVisibility,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

async fn database_pool() -> Option<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|value| !value.trim().is_empty())?;

    let pool = opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
        .ok()?;
    MIGRATOR.run(&pool).await.ok()?;
    Some(pool)
}

#[tokio::test]
async fn user_owned_repositories_enforce_uniqueness_permissions_and_pagination() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres repository scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        &pool,
        &format!("owner-{unique}@opengithub.local"),
        Some("Repository Owner"),
        None,
    )
    .await
    .expect("owner should upsert");
    let other_user = upsert_user_by_email(
        &pool,
        &format!("other-{unique}@opengithub.local"),
        Some("Other User"),
        None,
    )
    .await
    .expect("other user should upsert");

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: "alpha".to_owned(),
            description: Some("First repository".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("owner should create own repository");

    assert_eq!(repository.owner_user_id, Some(owner.id));
    assert_eq!(repository.owner_login, owner.username.as_deref().unwrap());
    assert_eq!(repository.visibility, RepositoryVisibility::Private);

    let permission = repository_permission_for_user(&pool, repository.id, owner.id)
        .await
        .expect("permission lookup should succeed")
        .expect("owner permission should be granted");
    assert_eq!(permission.role, RepositoryRole::Owner);
    assert!(permission.role.can_admin());

    let unauthorized = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: "blocked".to_owned(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: other_user.id,
        },
    )
    .await;
    assert!(
        unauthorized.is_err(),
        "a user must not create repositories under another user owner"
    );

    let duplicate = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: "ALPHA".to_owned(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: None,
            created_by_user_id: owner.id,
        },
    )
    .await;
    assert!(
        duplicate.is_err(),
        "repository names should be unique per owner, case-insensitively"
    );

    let list = list_repositories_for_user(&pool, owner.id, 1, 10)
        .await
        .expect("repository list should load");
    assert_eq!(list.total, 1);
    assert_eq!(list.page, 1);
    assert_eq!(list.page_size, 10);
    assert_eq!(list.items[0].id, repository.id);

    let fetched = get_repository_by_owner_name(&pool, &owner.email, "ALPHA")
        .await
        .expect("repository lookup should succeed")
        .expect("repository should be found case-insensitively");
    assert_eq!(fetched.id, repository.id);
}

#[tokio::test]
async fn organization_repositories_refs_commits_and_team_memberships_round_trip() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping Postgres repository scenario; set TEST_DATABASE_URL or DATABASE_URL");
        return;
    };

    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        &pool,
        &format!("org-owner-{unique}@opengithub.local"),
        Some("Org Owner"),
        None,
    )
    .await
    .expect("owner should upsert");

    let organization = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("open-org-{unique}"),
            display_name: "Open Org".to_owned(),
            description: Some("Organization for repository tests".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");

    let team_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO teams (organization_id, slug, name)
        VALUES ($1, 'core', 'Core')
        RETURNING id
        "#,
    )
    .bind(organization.id)
    .fetch_one(&pool)
    .await
    .expect("team should insert");
    sqlx::query(
        r#"
        INSERT INTO team_memberships (team_id, user_id, role)
        VALUES ($1, $2, 'maintainer')
        "#,
    )
    .bind(team_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("team membership should insert");

    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization {
                id: organization.id,
            },
            name: "platform".to_owned(),
            description: None,
            visibility: RepositoryVisibility::Public,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("org owner should create organization repository");
    assert_eq!(repository.owner_organization_id, Some(organization.id));
    assert_eq!(repository.owner_login, organization.slug);

    let commit = insert_commit(
        &pool,
        repository.id,
        CreateCommit {
            oid: format!("{}abcdef", unique.simple()),
            author_user_id: Some(owner.id),
            committer_user_id: Some(owner.id),
            message: "Initial commit".to_owned(),
            tree_oid: Some(format!("{}tree", unique.simple())),
            parent_oids: vec![],
            committed_at: Utc::now(),
        },
    )
    .await
    .expect("commit should insert");
    let git_ref = upsert_git_ref(
        &pool,
        repository.id,
        "refs/heads/trunk",
        "branch",
        Some(commit.id),
    )
    .await
    .expect("git ref should upsert");
    assert_eq!(git_ref.target_commit_id, Some(commit.id));

    let fetched = get_repository_by_owner_name(&pool, &organization.slug, "platform")
        .await
        .expect("repository lookup should succeed")
        .expect("organization repository should exist");
    assert_eq!(fetched.default_branch, "trunk");
}
