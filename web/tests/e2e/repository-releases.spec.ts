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
      const hasUploadLifecycle = execFileSync(
        "psql",
        [
          databaseUrl,
          "-tAc",
          "SELECT to_regclass('public.release_asset_upload_intents') IS NOT NULL",
        ],
        {
          env: {
            ...process.env,
            PGSSLMODE: process.env.PGSSLMODE ?? "disable",
          },
        },
      )
        .toString()
        .trim();
      if (hasUploadLifecycle !== "t") {
        execFileSync(
          "psql",
          [
            databaseUrl,
            "-v",
            "ON_ERROR_STOP=1",
            "-f",
            "../crates/api/migrations/202605031705_repository_release_asset_upload_lifecycle.up.sql",
          ],
          {
            env: {
              ...process.env,
              PGSSLMODE: process.env.PGSSLMODE ?? "disable",
            },
          },
        );
      }
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
  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605031705_repository_release_asset_upload_lifecycle.up.sql",
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
      ON CONFLICT (repository_id, name)
      DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id, kind = EXCLUDED.kind
      RETURNING repository_id
    ),
    branch AS (
      INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
      SELECT repo.id, 'refs/heads/main', 'branch', target.id
      FROM repo, target
      ON CONFLICT (repository_id, name)
      DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id, kind = EXCLUDED.kind
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

test("repository releases, dedicated management forms, and tags render seeded data", async ({
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
  await expect(
    page.getByRole("button", { exact: true, name: /rocket 0/ }),
  ).toBeVisible();
  await page.getByRole("button", { exact: true, name: "Compare" }).click();
  await expect(
    page.getByRole("textbox", { name: "Search branches and tags to compare" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-phase3-list-interactions.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-final-list.jpg",
  });

  await page.getByRole("link", { exact: true, name: "New release" }).click();
  await expect(page).toHaveURL(/\/releases\/new$/);
  await expect(
    page.getByRole("heading", { exact: true, name: "New release" }),
  ).toBeVisible();
  await expect(page.getByLabel("Existing tag")).toHaveValue("v2.0.0");
  await page
    .getByRole("textbox", { exact: true, name: "Title" })
    .fill("Managed release");
  await expect(
    page.getByRole("textbox", { exact: true, name: "Markdown source" }),
  ).toBeVisible();
  await page
    .getByRole("button", { exact: true, name: "Generate release notes" })
    .click();
  await expect(
    page.getByText("Generated notes inserted. Review them before publishing."),
  ).toBeVisible();
  await expect(
    page.getByRole("textbox", { exact: true, name: "Markdown source" }),
  ).not.toHaveValue("");
  await expect(page.getByLabel("Release asset files")).toBeEnabled();
  await expect(
    page.getByRole("button", { exact: true, name: "Publish release" }),
  ).toBeEnabled();
  await page.getByLabel("New tag").check();
  await page.getByLabel("New tag name").fill("v3.0.0");
  await page.getByRole("button", { exact: true, name: "Save draft" }).click();
  await expect(page).toHaveURL(/\/releases\/tag\/v3\.0\.0$/);
  await expect(
    page.getByRole("link", { exact: true, name: "Managed release" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-002-phase3-create-draft.jpg",
  });

  await page.goto(`${seeded.firstRepositoryHref}/releases/latest`);
  await expect(page).toHaveURL(/\/releases\/latest$/);
  await expect(page.getByText("Stable Editorial release")).toBeVisible();
  await page.getByRole("link", { exact: true, name: "Edit release" }).click();
  await expect(page).toHaveURL(/\/releases\/edit\//);
  await expect(
    page.getByRole("heading", { exact: true, name: "Edit release" }),
  ).toBeVisible();
  await expect(
    page.getByRole("textbox", { exact: true, name: "Title" }),
  ).toHaveValue("Stable Editorial release");
  await expect(
    page.getByText("opengithub.tar.gz", { exact: true }),
  ).toBeVisible();
  await page.getByLabel("Release asset files").setInputFiles({
    name: "manual-upload.zip",
    mimeType: "application/zip",
    buffer: Buffer.from("phase four asset"),
  });
  await expect(page.getByText("Attached to release.")).toBeVisible();
  await expect(page.getByText("manual-upload.zip")).toHaveCount(2);
  const uploadedAssetRow = page
    .locator("li", {
      hasText: "manual-upload.zip",
    })
    .last();
  await uploadedAssetRow.getByRole("button", { name: "Remove" }).click();
  await expect(page.getByText("Release asset removed.")).toBeVisible();
  await expect(
    page.locator("li", { hasText: "manual-upload.zip" }).last(),
  ).toContainText("complete");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-002-phase4-assets-edit.jpg",
  });
  await page
    .getByRole("textbox", { exact: true, name: "Title" })
    .fill("Stable Editorial release updated");
  await page
    .getByRole("button", { exact: true, name: "Update release" })
    .click();
  await expect(page).toHaveURL(/\/releases\/tag\/v2\.0\.0$/);
  await expect(
    page.getByText("Stable Editorial release updated"),
  ).toBeVisible();
  await page.goto(`${seeded.firstRepositoryHref}/releases/tag/v3.0.0`);
  await page.getByRole("link", { exact: true, name: "Edit release" }).click();
  await expect(
    page.getByRole("button", { exact: true, name: "Publish draft" }),
  ).toBeVisible();
  await page
    .getByRole("button", { exact: true, name: "Publish draft" })
    .click();
  await expect(page).toHaveURL(/\/releases\/tag\/v3\.0\.0$/);
  await page.getByRole("link", { exact: true, name: "Edit release" }).click();
  await expect(page.getByLabel("Also delete the git tag")).toBeVisible();
  await expect(
    page.getByRole("button", { exact: true, name: "Delete release" }),
  ).toBeDisabled();
  await page.getByLabel("Type tag name to confirm").fill("v3.0.0");
  await page
    .getByRole("button", { exact: true, name: "Delete release" })
    .click();
  await expect(page).toHaveURL(/\/releases$/);
  await expect(page.getByText("Managed release")).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-002-phase3-edit-publish-delete.jpg",
  });

  await page.goto(`${seeded.firstRepositoryHref}/releases/latest`);
  await expect(page).toHaveURL(/\/releases\/latest$/);
  await expect(
    page.getByText("Stable Editorial release updated"),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-002-phase2-detail-latest.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-final-detail-latest.jpg",
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
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-final-tags.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.firstRepositoryHref}/releases`);
  await expect(page.getByText("Stable Editorial release")).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/releases-001-final-mobile.jpg",
  });
});
