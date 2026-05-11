import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  requireTestDatabase,
  runPsql,
  screenshotPath,
  test,
} from "./_fixtures/auth";

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedDiscussions001(repositoryHref: string) {
  const databaseUrl = requireTestDatabase();
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");

  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      general AS (
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position)
        SELECT target_repo.id, 'general', 'General', '💬', 'General project conversation.', 1
        FROM target_repo
        RETURNING id, repository_id
      ),
      ideas AS (
        INSERT INTO discussion_categories (repository_id, slug, name, emoji, description, position)
        SELECT target_repo.id, 'ideas', 'Ideas', '💡', 'Shape product direction.', 2
        FROM target_repo
        RETURNING id, repository_id
      ),
      roadmap_label AS (
        INSERT INTO labels (repository_id, name, color, description)
        SELECT target_repo.id, 'roadmap', 'b85f36', 'Roadmap discussion'
        FROM target_repo
        RETURNING id, repository_id
      ),
      manifest_discussion AS (
        INSERT INTO discussions (repository_id, category_id, number, title, body, state, answered, author_user_id, comments_count, votes_count, last_activity_at)
        SELECT general.repository_id, general.id, 901,
               'How should repository import previews handle large manifests ${suffix}?',
               'Discuss manifest previews with **Markdown**.', 'open', true,
               target_repo.author_user_id, 8, 14, now() - interval '1 hour'
        FROM general, target_repo
        RETURNING id, repository_id
      ),
      idea_discussion AS (
        INSERT INTO discussions (repository_id, category_id, number, title, body, state, author_user_id, comments_count, votes_count, last_activity_at)
        SELECT ideas.repository_id, ideas.id, 902,
               'Empty-state CTA should preserve category context ${suffix}',
               'Discuss category route behavior.', 'open', target_repo.author_user_id, 1, 3, now() - interval '2 hours'
        FROM ideas, target_repo
        RETURNING id, repository_id
      ),
      closed_discussion AS (
        INSERT INTO discussions (repository_id, category_id, number, title, body, state, author_user_id, comments_count, votes_count, last_activity_at)
        SELECT general.repository_id, general.id, 903,
               'Closed roadmap archive ${suffix}', 'Closed roadmap body.', 'closed', target_repo.author_user_id, 2, 30, now() - interval '3 hours'
        FROM general, target_repo
        RETURNING id, repository_id
      )
      INSERT INTO discussion_labels (discussion_id, label_id)
      SELECT manifest_discussion.id, roadmap_label.id FROM manifest_discussion, roadmap_label
      UNION ALL
      SELECT closed_discussion.id, roadmap_label.id FROM closed_discussion, roadmap_label;

      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      manifest_discussion AS (
        SELECT discussions.id FROM discussions, target_repo
        WHERE discussions.repository_id = target_repo.id AND discussions.number = 901
        LIMIT 1
      )
      INSERT INTO discussion_pins (discussion_id, pinned_by_user_id, position)
      SELECT manifest_discussion.id, target_repo.author_user_id, 1
      FROM manifest_discussion, target_repo;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_community_links (repository_id, label, href, kind, position)
      SELECT target_repo.id, 'Code of conduct', ${sqlLiteral(`${repositoryHref}/community/code-of-conduct`)}, 'code_of_conduct', 1
      FROM target_repo;
    `,
  ]);
}

test("discussions-001 list category search filter sort and upvote behavior", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDiscussions001(repositoryHref);
  await signIn(page, seeded);

  await page.goto(`${repositoryHref}/discussions`);
  await expect(
    page.getByRole("heading", { name: "Discussions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Pinned discussions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /repository import previews/i }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /General/ }).last(),
  ).toHaveAttribute("href", /categories\/general/);
  await expect(page.getByText("Code of conduct")).toBeVisible();

  await page.getByLabel("discussion-query").fill("manifest");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/q=manifest/);
  await expect(
    page.getByRole("link", { name: /repository import previews/i }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Empty-state CTA/i }),
  ).toHaveCount(0);

  await page.goto(`${repositoryHref}/discussions?label=roadmap`);
  await expect(page).toHaveURL(/label=roadmap/);
  await expect(
    page.getByRole("link", { name: /repository import previews/i }).first(),
  ).toBeVisible();

  await page.goto(`${repositoryHref}/discussions?label=roadmap&sort=top`);
  await expect(page).toHaveURL(/sort=top/);
  await expect(page.locator("p", { hasText: "Sort: top" })).toBeVisible();

  await page
    .getByRole("link", { name: /General/ })
    .last()
    .click();
  await expect(page).toHaveURL(/\/discussions\/categories\/general/);
  await expect(page.getByText("category:general")).toBeVisible();

  const upvote = page.locator('button[aria-label*="discussion 901"]').first();
  await upvote.click();
  await expect(upvote).toHaveAttribute("aria-pressed", "true");
  await upvote.click();
  await expect(upvote).toHaveAttribute("aria-pressed", "false");

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "discussions-001-final-desktop"),
  });
  await page.setViewportSize({ width: 390, height: 900 });
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "discussions-001-final-mobile"),
  });
});
