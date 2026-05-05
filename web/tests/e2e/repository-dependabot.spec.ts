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

function seedDependabotAlerts(repositoryHref: string) {
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
      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      settings AS (
        INSERT INTO repository_security_feature_settings (
          repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        SELECT target_repo.id,
               'dependabot',
               'enabled',
               'Dependency alerts are monitored.',
               2,
               0,
               ${sqlLiteral(`${repositoryHref}/settings/security`)}
        FROM target_repo
        ON CONFLICT (repository_id, feature_key)
        DO UPDATE SET status = 'enabled',
                      summary = EXCLUDED.summary,
                      alert_count = 2,
                      private_count = 0,
                      config_href = EXCLUDED.config_href
      ),
      package_one AS (
        INSERT INTO dependency_packages (ecosystem, name, package_href)
        VALUES ('npm', '@playwright/test', '/packages/npm/%40playwright%2Ftest')
        ON CONFLICT (ecosystem, lower(name))
        DO UPDATE SET package_href = EXCLUDED.package_href
        RETURNING id
      ),
      package_two AS (
        INSERT INTO dependency_packages (ecosystem, name, package_href)
        VALUES ('cargo', 'sqlx', '/packages/cargo/sqlx')
        ON CONFLICT (ecosystem, lower(name))
        DO UPDATE SET package_href = EXCLUDED.package_href
        RETURNING id
      ),
      manifest_one AS (
        INSERT INTO dependency_manifests (
          repository_id, path, ecosystem, lockfile_path, dependency_count
        )
        SELECT target_repo.id, 'package.json', 'npm', 'package-lock.json', 1
        FROM target_repo
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET ecosystem = 'npm',
                      lockfile_path = 'package-lock.json',
                      dependency_count = 1
        RETURNING id, repository_id
      ),
      manifest_two AS (
        INSERT INTO dependency_manifests (
          repository_id, path, ecosystem, lockfile_path, dependency_count
        )
        SELECT target_repo.id, 'crates/api/Cargo.toml', 'cargo', NULL, 1
        FROM target_repo
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET ecosystem = 'cargo',
                      lockfile_path = NULL,
                      dependency_count = 1
        RETURNING id, repository_id
      ),
      dep_one AS (
        INSERT INTO repository_dependencies (
          repository_id, manifest_id, package_id, package_version, relationship, license, lockfile_path
        )
        SELECT manifest_one.repository_id,
               manifest_one.id,
               package_one.id,
               '1.55.0',
               'direct',
               'Apache-2.0',
               'package-lock.json'
        FROM manifest_one, package_one
        ON CONFLICT (manifest_id, package_id, relationship)
        DO UPDATE SET package_version = EXCLUDED.package_version,
                      lockfile_path = EXCLUDED.lockfile_path
        RETURNING package_id
      ),
      dep_two AS (
        INSERT INTO repository_dependencies (
          repository_id, manifest_id, package_id, package_version, relationship, license
        )
        SELECT manifest_two.repository_id,
               manifest_two.id,
               package_two.id,
               '0.8.0',
               'transitive',
               'MIT'
        FROM manifest_two, package_two
        ON CONFLICT (manifest_id, package_id, relationship)
        DO UPDATE SET package_version = EXCLUDED.package_version
        RETURNING package_id
      )
      INSERT INTO dependency_advisories (
        package_id, advisory_identifier, severity, title, advisory_href, published_at
      )
      SELECT dep_one.package_id,
             'GHSA-dependabot-${suffix}-one',
             'high',
             'Playwright test runner demo advisory',
             '/advisories/GHSA-dependabot-${suffix}-one',
             now() - interval '1 hour'
      FROM dep_one
      UNION ALL
      SELECT dep_two.package_id,
             'GHSA-dependabot-${suffix}-two',
             'moderate',
             'SQLx demo advisory',
             '/advisories/GHSA-dependabot-${suffix}-two',
             now() - interval '2 hours'
      FROM dep_two
      ON CONFLICT (package_id, advisory_identifier)
      DO UPDATE SET severity = EXCLUDED.severity,
                    title = EXCLUDED.title,
                    advisory_href = EXCLUDED.advisory_href;
      `,
    ],
    { stdio: "ignore" },
  );
}

function disableDependabot(repositoryHref: string) {
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
      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
          AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))}
        LIMIT 1
      )
      INSERT INTO repository_security_feature_settings (
        repository_id, feature_key, status, summary, alert_count, private_count, config_href
      )
      SELECT target_repo.id,
             'dependabot',
             'disabled',
             'Dependabot alerts are disabled for this repository.',
             0,
             0,
             ${sqlLiteral(`${repositoryHref}/settings/security`)}
      FROM target_repo
      ON CONFLICT (repository_id, feature_key)
      DO UPDATE SET status = 'disabled',
                    summary = EXCLUDED.summary,
                    alert_count = 0,
                    private_count = 0,
                    config_href = EXCLUDED.config_href;
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

test.skip(!databaseUrl, "repository Dependabot smoke needs a database URL");
test.setTimeout(90_000);

test("repository Dependabot alerts list supports filters, selection, disabled state, and screenshots", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedDependabotAlerts(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Dependabot alerts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Dependabot Dependency alerts and updates",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByText("Playwright test runner demo advisory"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "@playwright/test" }),
  ).toHaveAttribute("href", /\/security\/dependabot\/\d+/);
  await expect(
    page.getByRole("link", { name: "package.json" }),
  ).toHaveAttribute(
    "href",
    `${seeded.treeRepositoryHref}/blob/main/package.json`,
  );

  await page.getByRole("button", { name: "Package: All packages" }).click();
  await page.getByRole("menuitem", { name: /npm:@playwright\/test/ }).click();
  await expect(page).toHaveURL(/package=npm%3A%40playwright%2Ftest/);
  await expect(
    page.getByText("Playwright test runner demo advisory"),
  ).toBeVisible();

  await page.getByRole("button", { name: "Select all visible" }).click();
  await expect(
    page.getByRole("button", { name: "Clear visible" }),
  ).toBeVisible();
  await expect(
    page.getByText("Bulk triage arrives in the next phase"),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/code-security-002-phase2-alerts-list.jpg",
  });

  disableDependabot(seeded.treeRepositoryHref);
  await page.goto(`${seeded.treeRepositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Vulnerability alerts are disabled." }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open vulnerability settings" }),
  ).toHaveAttribute("href", `${seeded.treeRepositoryHref}/settings/security`);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.treeRepositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Dependabot alerts" }),
  ).toBeVisible();
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
});
