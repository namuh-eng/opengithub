import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  runPsql,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

const sqlLiteral = (value: string): string => `'${value.replace(/'/g, "''")}'`;

function seedProjectsListFixture(
  marker: string,
  orgHref: string,
  repositoryHref: string,
) {
  const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const org = orgHref.split("/")[2];
  const [, repoOwner, repoName] = repositoryHref.split("/");
  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
    DO $$
    DECLARE
      owner_id uuid;
      org_id uuid;
      repo_id uuid;
      org_repo_id uuid;
      org_project_id uuid;
      closed_project_id uuid;
      template_project_id uuid;
      repo_project_id uuid;
      user_project_id uuid;
      org_next bigint;
      user_next bigint;
    BEGIN
      SELECT users.id INTO owner_id
      FROM users
      WHERE lower(users.username) = lower(${sqlLiteral(decodeURIComponent(repoOwner))});

      SELECT organizations.id INTO org_id
      FROM organizations
      WHERE lower(organizations.slug) = lower(${sqlLiteral(decodeURIComponent(org))});

      SELECT repositories.id INTO repo_id
      FROM repositories
      LEFT JOIN users ON users.id = repositories.owner_user_id
      LEFT JOIN organizations repo_orgs ON repo_orgs.id = repositories.owner_organization_id
      WHERE lower(COALESCE(users.username, repo_orgs.slug)) = lower(${sqlLiteral(decodeURIComponent(repoOwner))})
        AND repositories.name = ${sqlLiteral(decodeURIComponent(repoName))};

      SELECT repositories.id INTO org_repo_id
      FROM repositories
      WHERE repositories.owner_organization_id = org_id
      ORDER BY repositories.created_at DESC
      LIMIT 1;

      IF owner_id IS NULL OR org_id IS NULL OR repo_id IS NULL OR org_repo_id IS NULL THEN
        RAISE EXCEPTION 'missing projects-list fixture anchors owner %, org %, repo %, org_repo %', owner_id, org_id, repo_id, org_repo_id;
      END IF;

      INSERT INTO organization_policy_settings (organization_id, projects_base_permission, projects_enabled)
      VALUES (org_id, 'write', true)
      ON CONFLICT (organization_id)
      DO UPDATE SET projects_base_permission = EXCLUDED.projects_base_permission,
                    projects_enabled = EXCLUDED.projects_enabled;

      SELECT COALESCE(max(number), 0) + 1 INTO org_next
      FROM projects
      WHERE owner_organization_id = org_id;

      INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, default_repository_id, created_by_user_id, updated_at)
      VALUES (org_id, org_next, ${sqlLiteral(`${marker} roadmap`)}, ${sqlLiteral("Tracks repository work and On track status for QA.")}, 'public', org_repo_id, owner_id, now())
      RETURNING id INTO org_project_id;

      INSERT INTO project_views (project_id, name, layout, position)
      VALUES (org_project_id, 'Table', 'table', 1), (org_project_id, 'Board', 'board', 2);
      INSERT INTO project_fields (project_id, name, field_type, position, settings)
      VALUES (org_project_id, 'Status', 'single_select', 1, '{"options":["Todo","Done"]}'::jsonb);
      INSERT INTO project_workflows (project_id, workflow_key, name, enabled, trigger_event)
      VALUES (org_project_id, 'auto-archive', 'Auto archive', true, 'item_closed');
      INSERT INTO project_items (project_id, item_type, title, position)
      VALUES (org_project_id, 'draft_issue', ${sqlLiteral(`${marker} draft`)}, 1);
      INSERT INTO project_repositories (project_id, repository_id, link_type)
      VALUES (org_project_id, org_repo_id, 'default')
      ON CONFLICT DO NOTHING;
      INSERT INTO project_status_updates (project_id, author_user_id, status, body)
      VALUES (org_project_id, owner_id, 'on_track', 'E2E status update');

      INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, state, created_by_user_id, updated_at, closed_at)
      VALUES (org_id, org_next + 1, ${sqlLiteral(`${marker} closed archive`)}, 'Closed scope for QA', 'public', 'closed', owner_id, now() - interval '1 day', now())
      RETURNING id INTO closed_project_id;

      INSERT INTO projects (owner_organization_id, number, title, short_description, visibility, is_template, created_by_user_id, updated_at)
      VALUES (org_id, org_next + 2, ${sqlLiteral(`${marker} template`)}, 'Reusable template for QA', 'public', true, owner_id, now() - interval '2 days')
      RETURNING id INTO template_project_id;
      INSERT INTO project_templates (project_id, title, description, is_public)
      VALUES (template_project_id, ${sqlLiteral(`${marker} template`)}, 'Copy this setup', true)
      ON CONFLICT (project_id) DO NOTHING;

      SELECT COALESCE(max(number), 0) + 1 INTO user_next
      FROM projects
      WHERE owner_user_id = owner_id;

      INSERT INTO projects (owner_user_id, number, title, short_description, visibility, default_repository_id, created_by_user_id, updated_at)
      VALUES (owner_id, user_next, ${sqlLiteral(`${marker} personal plan`)}, 'User-owned project for QA', 'public', repo_id, owner_id, now())
      RETURNING id INTO user_project_id;

      INSERT INTO projects (owner_user_id, number, title, short_description, visibility, default_repository_id, created_by_user_id, updated_at)
      VALUES (owner_id, user_next + 1, ${sqlLiteral(`${marker} repo linked`)}, 'Repository Projects tab linked by default repository', 'public', repo_id, owner_id, now())
      RETURNING id INTO repo_project_id;
      INSERT INTO project_repositories (project_id, repository_id, link_type)
      VALUES (repo_project_id, repo_id, 'linked')
      ON CONFLICT DO NOTHING;
    END $$;
    `,
  ]);
}

test.skip(
  skipWithoutTestDb(),
  "Projects list E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects v2 list pages filter, sort, switch tabs, and copy with real data", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  test.setTimeout(120_000);
  const seeded = await seed({ scenes: ["orgProfile"] });
  const marker = `projects-list-${Date.now()}`;
  seedProjectsListFixture(
    marker,
    seeded.hrefs.organizationProfile,
    seeded.hrefs.firstRepository,
  );
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.organizationProfile}/projects`);
  await expect(page.locator("#projects-list-title")).toContainText("projects");
  await expect(page.getByText("Welcome to Projects")).toBeVisible();
  await page
    .getByRole("textbox", { name: "Search all projects" })
    .fill(`${marker} roadmap`);
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page).toHaveURL(/q=projects-list-/);
  await expect(
    page.locator("article").filter({ hasText: `${marker} roadmap` }),
  ).toBeVisible();
  await expect(
    page.locator(".chip").filter({ hasText: "On track" }),
  ).toBeVisible();

  await page
    .getByRole("combobox", { name: "Sort projects" })
    .selectOption("name_asc");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page).toHaveURL(/sort=name_asc/);

  await page.goto(
    `${seeded.hrefs.organizationProfile}/projects?state=closed&q=${encodeURIComponent(
      `${marker} closed`,
    )}`,
  );
  await expect(
    page.locator("article").filter({ hasText: `${marker} closed archive` }),
  ).toBeVisible();
  await page.goto(
    `${seeded.hrefs.organizationProfile}/projects?tab=templates&q=${encodeURIComponent(
      `${marker} template`,
    )}`,
  );
  await expect(
    page.locator("article").filter({ hasText: `${marker} template` }),
  ).toBeVisible();

  await page.goto(
    `${seeded.hrefs.organizationProfile}/projects?q=${encodeURIComponent(`${marker} roadmap`)}`,
  );
  await page.getByText("More project options").first().click();
  await page.getByRole("menuitem", { name: "Copy project" }).click();
  await expect(
    page.getByRole("dialog", { name: `${marker} roadmap` }),
  ).toBeVisible();
  await expect(
    page.getByRole("textbox", { name: "Project title" }),
  ).toHaveValue(`[COPY] ${marker} roadmap`);
  await page.getByRole("button", { name: "Copy project" }).click();
  await expect(page).toHaveURL(/\/projects\/\d+\/views\/1/);

  await page.goto(
    `/${seeded.hrefs.firstRepository.split("/")[1]}?tab=projects&q=${encodeURIComponent(`${marker} personal`)}`,
  );
  await expect(page.getByText(`${marker} personal plan`)).toBeVisible();

  await page.goto(
    `${seeded.hrefs.firstRepository}/projects?q=${encodeURIComponent(`${marker} repo`)}`,
  );
  await expect(page.getByText(`${marker} repo linked`)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-001-final-list-pages"),
  });
});
