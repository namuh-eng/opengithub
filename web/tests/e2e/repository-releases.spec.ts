import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
};

function sqlLiteral(value: string) {
  return `'${value.replace(/'/g, "''")}'`;
}

function repositoryParts(firstRepositoryHref: string) {
  const [owner, repo] = firstRepositoryHref.split("/").filter(Boolean);
  if (!owner || !repo) {
    throw new Error(`Unexpected repository href: ${firstRepositoryHref}`);
  }
  return { owner, repo };
}

function runSql(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  execFileSync("psql", [databaseUrl, "-v", "ON_ERROR_STOP=1", "-c", sql], {
    env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
  });
}

function ensureReleaseReadSchema() {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const hasBodyHtml = execFileSync(
    "psql",
    [
      databaseUrl,
      "-tAc",
      "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'releases' AND column_name = 'body_html')",
    ],
    {
      env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
    },
  )
    .toString()
    .trim();
  if (hasBodyHtml === "t") {
    const hasManagementIndex = execFileSync(
      "psql",
      [
        databaseUrl,
        "-tAc",
        "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'releases_repository_tag_active_unique')",
      ],
      {
        env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
      },
    )
      .toString()
      .trim();
    if (hasManagementIndex === "t") {
      return;
    }
    execFileSync(
      "psql",
      [
        databaseUrl,
        "-v",
        "ON_ERROR_STOP=1",
        "-f",
        "../crates/api/migrations/202605031328_repository_releases_management.up.sql",
      ],
      {
        env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
      },
    );
    return;
  }
  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605030047_repository_releases_read.up.sql",
    ],
    {
      env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
    },
  );
  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605031328_repository_releases_management.up.sql",
    ],
    {
      env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
    },
  );
}

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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
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

function seedRelease(firstRepositoryHref: string) {
  ensureReleaseReadSchema();
  const { owner, repo } = repositoryParts(firstRepositoryHref);
  runSql(`
    WITH repo AS (
      SELECT repositories.id, repositories.owner_user_id
      FROM repositories
      JOIN users ON users.id = repositories.owner_user_id
      WHERE users.username = ${sqlLiteral(owner)}
        AND repositories.name = ${sqlLiteral(repo)}
    ),
    target AS (
      SELECT commits.id, commits.oid
      FROM commits
      WHERE repository_id = (SELECT id FROM repo)
      ORDER BY committed_at DESC NULLS LAST, created_at DESC
      LIMIT 1
    ),
    tag AS (
      INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
      SELECT repo.id, 'refs/tags/v2.0.0', 'tag', target.id
      FROM repo, target
      RETURNING repository_id
    ),
    release AS (
      INSERT INTO releases (
        repository_id, tag_name, name, body, body_html, rendered_body_excerpt,
        draft, prerelease, is_latest, tag_verified, tag_signature_summary,
        author_user_id, target_commit_id, published_at, created_at
      )
      SELECT repo.id,
             'v2.0.0',
             'Stable Editorial release',
             '## Highlights',
             '<h2>Highlights</h2><p>Safe release notes.</p>',
             '<p>Safe release notes.</p>',
             false,
             false,
             true,
             true,
             'Verified tag signature',
             repo.owner_user_id,
             target.id,
             now(),
             now()
      FROM repo, target
      RETURNING id, repository_id
    )
    INSERT INTO release_assets (
      repository_id, release_id, name, label, content_type, byte_size,
      storage_key, download_count, uploaded_by_user_id
    )
    SELECT release.repository_id,
           release.id,
           'opengithub.tar.gz',
           'Linux build',
           'application/gzip',
           2048,
           'releases/e2e/opengithub.tar.gz',
           42,
           repo.owner_user_id
    FROM release, repo;
  `);
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
    if (await button.isDisabled()) {
      await expect(button).toHaveAttribute("aria-disabled", "true");
    }
  }
}

test.skip(
  !databaseUrl,
  "repository Releases smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("repository releases, latest detail, and tags render seeded read data", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedRelease(seeded.firstRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/releases`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Releases" }),
  ).toBeVisible();
  await expect(page.getByText("Stable Editorial release")).toBeVisible();
  await expect(page.getByText("Latest", { exact: true })).toBeVisible();
  await expect(page.getByText("Verified", { exact: true })).toBeVisible();
  await page.getByText("Assets 3").click();
  await expect(page.getByText("opengithub.tar.gz")).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "opengithub.tar.gz" }),
  ).toHaveAttribute("href", /\/releases\/assets\//);
  await page.getByRole("button", { exact: true, name: /rocket 0/ }).click();
  await expect(page.getByRole("status")).toContainText("Reaction updated");
  await expect(
    page.getByRole("button", { exact: true, name: /rocket 1/ }),
  ).toBeVisible();
  await page.getByRole("button", { exact: true, name: "Compare" }).click();
  await expect(
    page.getByRole("textbox", { name: "Search branches and tags to compare" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /main/ }).first(),
  ).toHaveAttribute("href", /\/compare\/v2\.0\.0\.\.\.main/);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-phase3-list-interactions.jpg",
  });

  await page.getByRole("button", { exact: true, name: "New release" }).click();
  await page.getByRole("textbox", { exact: true, name: "Tag" }).fill("v2.0.1");
  await page
    .getByRole("textbox", { exact: true, name: "Title" })
    .fill("Managed release");
  await page
    .getByRole("textbox", { exact: true, name: "Notes" })
    .fill("Managed notes from browser smoke.");
  await page.getByLabel("Save as draft").check();
  await page
    .getByRole("button", { exact: true, name: "Create release" })
    .click();
  await expect(page).toHaveURL(/\/releases\/tag\/v2\.0\.1$/);
  await expect(page.getByText("Managed release")).toBeVisible();
  await page.getByRole("button", { exact: true, name: "Edit" }).click();
  await expect(
    page.getByRole("button", { exact: true, name: "Publish draft" }),
  ).toBeVisible();
  await page.getByLabel("Asset name").fill("opengithub-browser.tar.gz");
  await page.getByLabel("Asset label").fill("Browser smoke");
  await page.getByLabel("Asset byte size").fill("4096");
  await page.getByRole("button", { exact: true, name: "Upload asset" }).click();
  await expect(page.getByText("Asset added.")).toBeVisible();
  await expect(
    page
      .getByLabel("Release management")
      .getByText("opengithub-browser.tar.gz"),
  ).toBeVisible();
  await page
    .getByRole("button", { exact: true, name: "Publish draft" })
    .click();
  await expect(page.getByText("Draft published.")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-phase4-management.jpg",
  });

  await page.goto(`${seeded.firstRepositoryHref}/releases/latest`);
  await expect(page).toHaveURL(/\/releases\/latest$/);
  await expect(page.getByText("Managed release")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-phase3-detail-latest.jpg",
  });

  await page.goto(`${seeded.firstRepositoryHref}/tags`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Tags" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "v2.0.0" })).toBeVisible();
  await expect(
    page.locator(
      `a[href="${seeded.firstRepositoryHref}/releases/tag/v2.0.0"]`,
      {
        hasText: "Release",
      },
    ),
  ).toHaveAttribute(
    "href",
    `${seeded.firstRepositoryHref}/releases/tag/v2.0.0`,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-phase3-tags.jpg",
  });
});
