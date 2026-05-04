import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  treeRepositoryHref: string;
};

function seedDashboard(): SeededDashboard {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }

  const output = execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "-p",
      "opengithub-api",
      "--example",
      "dashboard_e2e_seed",
    ],
    {
      cwd: "..",
      env: {
        ...process.env,
        DASHBOARD_E2E_EMPTY: "0",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        DASHBOARD_E2E_TREE_REFS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedNetwork(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");
  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-c",
      `
      WITH source AS (
        SELECT repositories.id, repositories.created_by_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      owner_one AS (
        INSERT INTO users (email, username, display_name)
        VALUES ('network-one-${suffix}@opengithub.local', 'network-one-${suffix}', 'Network One')
        RETURNING id, username
      ),
      owner_two AS (
        INSERT INTO users (email, username, display_name)
        VALUES ('network-two-${suffix}@opengithub.local', 'network-two-${suffix}', 'Network Two')
        RETURNING id, username
      ),
      fork_one AS (
        INSERT INTO repositories (
          owner_user_id, name, description, visibility, default_branch, created_by_user_id, created_at, updated_at
        )
        SELECT owner_one.id, 'network-active-${suffix}', 'Active fork seeded for the Network smoke.', 'public', 'release/main', owner_one.id, now() - interval '8 days', now() - interval '1 day'
        FROM owner_one
        RETURNING id, owner_user_id
      ),
      fork_two AS (
        INSERT INTO repositories (
          owner_user_id, name, description, visibility, default_branch, is_archived, created_by_user_id, created_at, updated_at
        )
        SELECT owner_two.id, 'network-archived-${suffix}', null, 'public', 'main', true, owner_two.id, now() - interval '30 days', now() - interval '12 days'
        FROM owner_two
        RETURNING id, owner_user_id
      ),
      edges AS (
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id, created_at)
        SELECT source.id, fork_one.id, fork_one.owner_user_id, now() - interval '8 days' FROM source, fork_one
        UNION ALL
        SELECT source.id, fork_two.id, fork_two.owner_user_id, now() - interval '30 days' FROM source, fork_two
        ON CONFLICT DO NOTHING
        RETURNING fork_repository_id
      ),
      commit_one AS (
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, tree_oid, committed_at)
        SELECT fork_one.id, 'network-active-${suffix}-commit', fork_one.owner_user_id, fork_one.owner_user_id, 'Push active fork', 'network-active-${suffix}-tree', now() - interval '1 day'
        FROM fork_one
        ON CONFLICT (repository_id, oid) DO UPDATE SET committed_at = EXCLUDED.committed_at
        RETURNING id, repository_id
      ),
      commit_two AS (
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message, tree_oid, committed_at)
        SELECT fork_two.id, 'network-archived-${suffix}-commit', fork_two.owner_user_id, fork_two.owner_user_id, 'Push archived fork', 'network-archived-${suffix}-tree', now() - interval '12 days'
        FROM fork_two
        ON CONFLICT (repository_id, oid) DO UPDATE SET committed_at = EXCLUDED.committed_at
        RETURNING id, repository_id
      )
      INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
      SELECT repository_id, 'refs/heads/release/main', 'branch', id FROM commit_one
      UNION ALL
      SELECT repository_id, 'refs/heads/main', 'branch', id FROM commit_two
      ON CONFLICT (repository_id, name)
      DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id;

      WITH fork_one AS (
        SELECT repositories.id, repositories.created_by_user_id
        FROM repositories
        JOIN users ON users.id = repositories.owner_user_id
        WHERE users.username = 'network-one-${suffix}'
          AND repositories.name = 'network-active-${suffix}'
      ),
      fork_two AS (
        SELECT repositories.id, repositories.created_by_user_id
        FROM repositories
        JOIN users ON users.id = repositories.owner_user_id
        WHERE users.username = 'network-two-${suffix}'
          AND repositories.name = 'network-archived-${suffix}'
      ),
      source AS (
        SELECT repositories.created_by_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_stars (user_id, repository_id)
      SELECT created_by_user_id, id FROM fork_one
      UNION ALL
      SELECT created_by_user_id, id FROM fork_two
      UNION ALL
      SELECT source.created_by_user_id, fork_two.id FROM source, fork_two
      ON CONFLICT DO NOTHING;
      `,
    ],
    { stdio: "ignore" },
  );
}

async function signIn(page: Page, seeded: SeededDashboard) {
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: seeded.cookieValue,
    },
  ]);
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test.skip(!databaseUrl, "repository Network smoke needs a database URL");
test.setTimeout(90_000);

test("repository Network renders readable fork graph and concrete links", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedNetwork(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/network`);
  await expect(
    page.getByRole("heading", { name: "Repository network" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Network Repository network activity",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.locator(".chip", { hasText: "Fresh projection" }),
  ).toBeVisible();
  await expect(page.getByLabel("Network summary metrics")).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Repository network fork graph" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: /^network-one-.*\/network-active-[a-f0-9]+$/,
    }),
  ).toHaveAttribute("href", /\/network-one-.*\/network-active-/);
  await expect(
    page.getByRole("link", { name: /network-one-.*\/network-active-.* tree/ }),
  ).toHaveAttribute("href", /\/tree\/release%2Fmain$/);
  await expect(
    page.getByRole("link", {
      name: /network-one-.*\/network-active-.* network/,
    }),
  ).toHaveAttribute("href", /\/network$/);
  await expect(
    page.getByRole("link", { name: /1 stars/ }).first(),
  ).toBeVisible();
  await expect(page.locator(".chip", { hasText: /^archived$/ })).toBeVisible();
  await expect(page.getByRole("link", { name: "View forks" })).toHaveAttribute(
    "href",
    `${seeded.treeRepositoryHref}/forks`,
  );
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-004-final-network-desktop.jpg",
  });

  await page.getByRole("link", { name: "View forks" }).click();
  await expect(
    page.getByRole("heading", { name: "Forked repositories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Forks Forked repositories" }),
  ).toHaveAttribute("aria-current", "page");
  await page.getByRole("button", { name: "Period: Last month" }).click();
  await page.getByRole("menuitem", { name: /All time/ }).click();
  await expect(page).toHaveURL(
    /\/forks\?period=all&type=all&sort=most_starred/,
  );
  await page
    .getByRole("button", { name: "Repository type: All repositories" })
    .click();
  await page.getByRole("menuitem", { name: /Starred by you/ }).click();
  await expect(page).toHaveURL(/type=starred/);
  await page.getByRole("button", { name: "Sort: Most starred" }).click();
  await page.getByRole("menuitem", { name: /Recently pushed/ }).click();
  await expect(page).toHaveURL(/sort=recently_pushed/);
  await expect(
    page.getByRole("list", { name: "Repository forks list" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Save defaults" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Save defaults" }).click();
  await expect(page.getByText("Saved for this repository")).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: /network-two-.*\/network-archived-.* tree/,
    }),
  ).toHaveAttribute("href", /\/tree\/main$/);
  await expect(
    page.getByRole("link", { name: "Switch to tree view" }),
  ).toHaveAttribute("href", /\/tree\//);

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-004-phase4-edge-cases.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-004-final-forks-desktop.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Forked repositories" }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-004-final-mobile.jpg",
  });
});
