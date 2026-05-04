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
        DASHBOARD_E2E_TREE_REFS: "1",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedContributorFileChanges(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-c",
      `
      INSERT INTO commit_file_changes (commit_id, path, status, additions, deletions)
      SELECT commits.id, 'src/contributors.rs', 'modified', 18, 4
      FROM commits
      JOIN repositories ON repositories.id = commits.repository_id
      LEFT JOIN users ON users.id = repositories.owner_user_id
      LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
      WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
        AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))}
        AND commits.committed_at >= now() - interval '1 week'
      ON CONFLICT (commit_id, path)
      DO UPDATE SET additions = EXCLUDED.additions, deletions = EXCLUDED.deletions;
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

test.skip(!databaseUrl, "repository Contributors smoke needs a database URL");
test.setTimeout(90_000);

test("repository Contributors renders default analytics and concrete drilldowns", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedContributorFileChanges(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/graphs/contributors`);
  await expect(
    page.getByRole("heading", { name: "Contributor analytics" }),
  ).toBeVisible();
  await expect(page.getByText(/Default branch scope:/)).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Contributors Contributor commit activity",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("button", { name: "Period: Last week" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Repository commits over time chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Repository contributors data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "View as data table" }).first(),
  ).toHaveAttribute("href", "#contributors-data-table");
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);
  await expect(
    page.getByRole("link", { name: /\d+ commits/ }).first(),
  ).toHaveAttribute("href", /\/commits\/main\?.*author=/);
  await expect(
    page.getByRole("link", { name: /pulse-e2e|dashboard/ }).first(),
  ).toHaveAttribute("href", /^\/[^/]+$/);

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-002-phase2-contributors-overview.jpg",
  });

  await page.getByRole("button", { name: "Period: Last week" }).click();
  await expect(
    page.getByRole("menu", { name: "Contributors period" }),
  ).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "Last 24 hours" }),
  ).toHaveAttribute("href", /\/graphs\/contributors\?period=24h$/);
  await expect(
    page.getByRole("menuitem", { name: "Last month" }),
  ).toHaveAttribute("href", /\/graphs\/contributors\?period=1m$/);
  await page.getByRole("menuitem", { name: "Last 3 days" }).click();
  await expect(page).toHaveURL(/\/graphs\/contributors\?period=3d$/);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Repository contributors data table" }),
  ).toBeVisible();
});
