import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
};

type SeedPackageOptions = {
  visibility?: "public" | "private" | "internal";
  packageType?: string;
  prefix?: string;
};

function sqlLiteral(value: string) {
  return `'${value.replace(/'/g, "''")}'`;
}

function execPsql(args: string[]) {
  const env = { ...process.env, PGSSLMODE: process.env.PGSSLMODE ?? "disable" };
  try {
    return execFileSync("psql", args, { env });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") {
      throw error;
    }
  }

  if (!databaseUrl?.includes("localhost:55433/opengithub_test")) {
    throw new Error(
      "psql is not installed and Docker fallback only supports the test DB",
    );
  }

  const dockerArgs = [
    "exec",
    "-i",
    "opengithub-postgres-test",
    "psql",
    "-U",
    "opengithub",
    "-d",
    "opengithub_test",
  ];
  const psqlArgs = args[0] === databaseUrl ? args.slice(1) : args;
  let input: Buffer | undefined;
  for (let index = 0; index < psqlArgs.length; index += 1) {
    const arg = psqlArgs[index];
    if (arg === "-f") {
      const file = psqlArgs[index + 1];
      if (!file) {
        throw new Error("psql -f requires a file");
      }
      input = readFileSync(file);
      dockerArgs.push("-f", "-");
      index += 1;
    } else {
      dockerArgs.push(arg);
    }
  }

  return execFileSync("docker", dockerArgs, { input });
}

function runSql(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  execPsql([databaseUrl, "-v", "ON_ERROR_STOP=1", "-c", sql]);
}

function queryScalar(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  return execPsql([databaseUrl, "-tAc", sql]).toString().trim();
}

function ensurePackageDetailSchema() {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const hasPackageDownloads = execPsql([
    databaseUrl,
    "-tAc",
    "SELECT to_regclass('public.package_downloads') IS NOT NULL",
  ])
    .toString()
    .trim();
  if (hasPackageDownloads !== "t") {
    execPsql([
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605030041_owner_packages.up.sql",
    ]);
  }
  const hasPackageBlobs = execPsql([
    databaseUrl,
    "-tAc",
    "SELECT to_regclass('public.package_blobs') IS NOT NULL",
  ])
    .toString()
    .trim();
  if (hasPackageBlobs !== "t") {
    execPsql([
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605031930_package_detail_metadata.up.sql",
    ]);
  }
  const hasPackageRegistryAudit = execPsql([
    databaseUrl,
    "-tAc",
    "SELECT to_regclass('public.package_registry_audit_events') IS NOT NULL",
  ])
    .toString()
    .trim();
  if (hasPackageRegistryAudit !== "t") {
    execPsql([
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605032020_package_registry_manifest_reads.up.sql",
    ]);
    execPsql([
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605032230_package_registry_actions_publishing.up.sql",
    ]);
  }
  const hasPackageDeletedAt = execPsql([
    databaseUrl,
    "-tAc",
    "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'packages' AND column_name = 'deleted_at')",
  ])
    .toString()
    .trim();
  if (hasPackageDeletedAt !== "t") {
    execPsql([
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-f",
      "../crates/api/migrations/202605032345_package_admin_lifecycle.up.sql",
    ]);
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

function seedPackage(
  firstRepositoryHref: string,
  {
    visibility = "public",
    packageType = "container",
    prefix = "detail",
  }: SeedPackageOptions = {},
) {
  ensurePackageDetailSchema();
  const { owner, repo } = repositoryParts(firstRepositoryHref);
  const packageName = `${prefix}-${Date.now()}`;
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
             ${sqlLiteral(packageName)}, ${sqlLiteral(packageType)}, ${sqlLiteral(visibility)}
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
  return `/${owner}/${packageType}/${packageName}`;
}

function seedOrganizationPackage(firstRepositoryHref: string) {
  ensurePackageDetailSchema();
  const { owner } = repositoryParts(firstRepositoryHref);
  const marker = `pkgorg${Date.now()}`;
  const packageName = `${marker}-web`;
  runSql(`
    WITH actor AS (
      SELECT users.id AS user_id
      FROM users
      WHERE users.username = ${sqlLiteral(owner)}
      LIMIT 1
    ),
    org AS (
      INSERT INTO organizations (slug, display_name, description, owner_user_id)
      SELECT ${sqlLiteral(marker)}, 'Package Detail Final Org',
             'Organization package detail final smoke', user_id
      FROM actor
      RETURNING id, owner_user_id, slug
    ),
    membership AS (
      INSERT INTO organization_memberships (organization_id, user_id, role)
      SELECT id, owner_user_id, 'owner'
      FROM org
      ON CONFLICT (organization_id, user_id) DO UPDATE SET role = EXCLUDED.role
      RETURNING organization_id
    ),
    source_repo AS (
      INSERT INTO repositories (
        owner_user_id, owner_organization_id, name, description, visibility,
        default_branch, created_by_user_id
      )
      SELECT NULL, org.id, ${sqlLiteral(`${marker}-repo`)},
             'Org package source', 'internal', 'main', org.owner_user_id
      FROM org
      RETURNING id, owner_organization_id, created_by_user_id
    ),
    package AS (
      INSERT INTO packages (
        repository_id, owner_user_id, owner_organization_id, created_by_user_id,
        name, package_type, visibility
      )
      SELECT id, NULL, owner_organization_id, created_by_user_id,
             ${sqlLiteral(packageName)}, 'npm', 'internal'
      FROM source_repo
      RETURNING id, created_by_user_id
    ),
    version AS (
      INSERT INTO package_versions (
        package_id, version, digest, platform_os, platform_arch, size_bytes,
        published_by_user_id, readme_markdown
      )
      SELECT id, '3.1.0',
             'sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
             'linux', 'amd64', 8192, created_by_user_id,
             '# Org Package README'
      FROM package
      RETURNING id, package_id
    ),
    about AS (
      INSERT INTO package_about_overrides (package_id, markdown, updated_by_user_id)
      SELECT id, '# Org Package README', created_by_user_id
      FROM package
      RETURNING package_id
    )
    INSERT INTO package_downloads (package_id, package_version_id, download_count)
    SELECT version.package_id, version.id, 9
    FROM version
    JOIN about ON about.package_id = version.package_id;
  `);
  return `/orgs/${marker}/packages/npm/${packageName}`;
}

test.skip(
  !databaseUrl,
  "package detail E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("package detail final smoke covers user, org, settings, forbidden, and mobile states", async ({
  page,
}) => {
  test.setTimeout(90_000);

  runSql("DELETE FROM rate_limit_buckets");
  await expect
    .poll(
      async () => {
        try {
          const response = await page.request.get(
            "http://localhost:3016/health",
            {
              headers: { "X-Forwarded-For": "10.201.0.1" },
              timeout: 1000,
            },
          );
          return response.status();
        } catch {
          return 0;
        }
      },
      { timeout: 60_000 },
    )
    .toBe(200);

  const seeded = seedDashboard();
  await signIn(page, seeded);
  const packageHref = seedPackage(seeded.firstRepositoryHref);
  const privatePackageHref = seedPackage(seeded.firstRepositoryHref, {
    prefix: "private-detail",
    visibility: "private",
  });
  const orgPackageHref = seedOrganizationPackage(seeded.firstRepositoryHref);
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
    { headers: { "X-Forwarded-For": "10.201.0.2" } },
  );
  expect(metadataResponse.ok()).toBe(true);
  const metadata = await metadataResponse.json();
  expect(metadata.downloadCount).toBe(downloadsAfterRender + 1);
  await expect(
    page.getByRole("heading", { exact: true, name: "README" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Settings" })).toHaveAttribute(
    "href",
    /\/settings$/,
  );
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-final-user-detail.jpg",
  });

  await page.getByRole("link", { name: "2.0.0" }).click();
  await expect(page).toHaveURL(/version=sha256%3Abbbbb/);
  await expect(
    page.getByText(/docker pull ghcr\.io\/.*:2\.0\.0@sha256:bbbb/),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-final-version-detail.jpg",
  });

  await page.getByRole("link", { name: "Settings" }).click();
  await expect(page.getByText("Package settings")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Explicit package access" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Linked repositories" }),
  ).toBeVisible();
  await expect(page.getByText("Enabled").first()).toBeVisible();
  await expect(page.getByRole("button", { name: "Grant" })).toBeDisabled();
  await page.getByLabel("Package visibility").selectOption("private");
  await page.getByRole("button", { name: "Save visibility" }).click();
  await expect(page.getByText("Package visibility saved.")).toBeVisible();
  await page.getByRole("button", { name: "Delete package" }).click();
  await expect(page.getByText("Package soft-deleted.")).toBeVisible();
  await expect(page.getByText("Deleted", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Restore package" }).click();
  await expect(page.getByText("Package restored.")).toBeVisible();
  await expect(page.locator("body")).not.toContainText(
    "redacted-package-layer",
  );
  await expect(page.locator("body")).not.toContainText("s3://");
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-003-phase4-settings.jpg",
  });

  await page.goto(orgPackageHref);
  await expect(
    page.getByRole("heading", { name: /pkgorg.*-web/ }),
  ).toBeVisible();
  await expect(page.getByText("internal")).toBeVisible();
  await expect(
    page.getByRole("heading", { exact: true, name: "README" }),
  ).toBeVisible();
  await expect(page.getByText("Org Package README")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-final-org-detail.jpg",
  });

  await page.context().clearCookies();
  await page.goto(privatePackageHref);
  await expect(
    page.getByRole("heading", { name: "Package could not load" }),
  ).toBeVisible();
  await expect(page.locator("body")).not.toContainText("private-detail");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-final-forbidden.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await signIn(page, seeded);
  await page.goto(`${packageHref}`);
  const mobileOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(mobileOverflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/packages-002-final-mobile.jpg",
  });
});
