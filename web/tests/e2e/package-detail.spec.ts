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

function runSql(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  execFileSync("psql", [databaseUrl, "-v", "ON_ERROR_STOP=1", "-c", sql], {
    env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
  });
}

function queryScalar(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  return execFileSync("psql", [databaseUrl, "-tAc", sql], {
    env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
  })
    .toString()
    .trim();
}

function ensurePackageDetailSchema() {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const hasPackageDownloads = execFileSync(
    "psql",
    [
      databaseUrl,
      "-tAc",
      "SELECT to_regclass('public.package_downloads') IS NOT NULL",
    ],
    {
      env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
    },
  )
    .toString()
    .trim();
  if (hasPackageDownloads !== "t") {
    execFileSync(
      "psql",
      [
        databaseUrl,
        "-v",
        "ON_ERROR_STOP=1",
        "-f",
        "../crates/api/migrations/202605030041_owner_packages.up.sql",
      ],
      {
        env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
      },
    );
  }
  const hasPackageBlobs = execFileSync(
    "psql",
    [
      databaseUrl,
      "-tAc",
      "SELECT to_regclass('public.package_blobs') IS NOT NULL",
    ],
    {
      env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
    },
  )
    .toString()
    .trim();
  if (hasPackageBlobs !== "t") {
    execFileSync(
      "psql",
      [
        databaseUrl,
        "-v",
        "ON_ERROR_STOP=1",
        "-f",
        "../crates/api/migrations/202605031930_package_detail_metadata.up.sql",
      ],
      {
        env: { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" },
      },
    );
  }
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

function repositoryParts(firstRepositoryHref: string) {
  const [owner, repo] = firstRepositoryHref.split("/").filter(Boolean);
  if (!owner || !repo) {
    throw new Error(`Unexpected repository href: ${firstRepositoryHref}`);
  }
  return { owner, repo };
}

function seedPackage(firstRepositoryHref: string) {
  ensurePackageDetailSchema();
  const { owner, repo } = repositoryParts(firstRepositoryHref);
  const packageName = `detail-${Date.now()}`;
  runSql(`
    WITH repo AS (
      SELECT repositories.id AS repository_id, repositories.owner_user_id AS owner_user_id
      FROM repositories
      JOIN users ON users.id = repositories.owner_user_id
      WHERE users.username = ${sqlLiteral(owner)}
        AND repositories.name = ${sqlLiteral(repo)}
      LIMIT 1
    ),
    package AS (
      INSERT INTO packages (
        repository_id, owner_user_id, owner_organization_id, created_by_user_id,
        name, package_type, visibility
      )
      SELECT repository_id, owner_user_id, NULL, owner_user_id,
             ${sqlLiteral(packageName)}, 'container', 'public'
      FROM repo
      RETURNING id, created_by_user_id
    ),
    older_version AS (
      INSERT INTO package_versions (
        package_id, version, digest, platform_os, platform_arch, size_bytes,
        published_by_user_id, created_at
      )
      SELECT id, '1.0.0',
             'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
             'linux', 'amd64', 2048, created_by_user_id, now() - interval '1 day'
      FROM package
      RETURNING id, package_id
    ),
    latest_version AS (
      INSERT INTO package_versions (
        package_id, version, digest, platform_os, platform_arch, size_bytes,
        published_by_user_id
      )
      SELECT id, '2.0.0',
             'sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
             'linux', 'arm64', 4096, created_by_user_id
      FROM package
      RETURNING id, package_id
    ),
    blobs AS (
      INSERT INTO package_blobs (
        package_id, package_version_id, digest, media_type, platform_os,
        platform_arch, size_bytes, storage_key
      )
      SELECT package_id, id,
             'sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
             'application/vnd.oci.image.layer.v1.tar+gzip', 'linux', 'arm64',
             4096, 's3://redacted-package-layer'
      FROM latest_version
      RETURNING package_id
    )
    INSERT INTO package_downloads (package_id, package_version_id, download_count)
    SELECT package_id, id, 27
    FROM latest_version;
  `);
  return `/${owner}/container/${packageName}`;
}

test.skip(
  !databaseUrl,
  "package detail E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("package detail renders install, versions, about, and mobile-safe layout", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const packageHref = seedPackage(seeded.firstRepositoryHref);
  const packageName = packageHref.split("/").at(-1) ?? "";

  await page.goto(`${packageHref}?version=2.0.0`);
  await expect(page.getByRole("heading", { name: /detail-/ })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Install from the command line" }),
  ).toBeVisible();
  await expect(page.getByText(/docker pull ghcr\.io\//).first()).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Recent versions" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "2.0.0" })).toHaveAttribute(
    "href",
    /version=sha256%3Abbbbb/,
  );
  await expect(page.getByRole("link", { name: "1.0.0" })).toHaveAttribute(
    "href",
    /version=sha256%3Aaaaaa/,
  );
  await page.getByLabel("Package version").selectOption({ label: "1.0.0" });
  await expect(
    page.getByText(/docker pull ghcr\.io\/.*:1\.0\.0@sha256:aaaaa/),
  ).toBeVisible();
  await page.getByRole("button", { name: "Copy install command" }).click();
  await expect(page.getByText("Command copied")).toBeVisible();
  await page.getByText("Pull this immutable digest").click();
  await expect(
    page
      .locator("details")
      .filter({ hasText: "Pull this immutable digest" })
      .getByText(/docker pull ghcr\.io\/[^:]+@sha256:aaaaaaaa/),
  ).toBeVisible();

  const downloadsAfterRender = Number(
    queryScalar(
      `SELECT COALESCE(SUM(pd.download_count), 0)::bigint FROM package_downloads pd JOIN packages p ON p.id = pd.package_id WHERE p.name = ${sqlLiteral(packageName)}`,
    ),
  );
  expect(downloadsAfterRender).toBe(27);
  const metadataResponse = await page.request.get(
    `http://localhost:3016/api/users/${packageHref.split("/")[1]}/packages/container/${packageName}/download?version=1.0.0`,
  );
  expect(metadataResponse.ok()).toBe(true);
  const metadata = await metadataResponse.json();
  expect(metadata.downloadCount).toBe(downloadsAfterRender + 1);
  await expect(page.getByRole("heading", { name: "README" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Settings" })).toHaveAttribute(
    "href",
    /\/settings$/,
  );
  await page.getByRole("link", { name: "Settings" }).click();
  await expect(page.getByText("Package settings")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Explicit package access" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Linked repositories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Not available" }).first(),
  ).toHaveAttribute("aria-disabled", "true");
  await expect(page.getByText(/packages-003/).first()).toBeVisible();
  await expect(page.locator("body")).not.toContainText(
    "redacted-package-layer",
  );
  await expect(page.locator("body")).not.toContainText("s3://");
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-phase4-settings.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${packageHref}/settings`);
  const mobileOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(mobileOverflow).toBe(false);
});
