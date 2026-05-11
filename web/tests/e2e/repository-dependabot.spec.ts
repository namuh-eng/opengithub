import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  requireTestDatabase,
  runPsql,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

function sqlLiteral(value: string) {
  return `'${value.replace(/'/g, "''")}'`;
}

function seedDependabotAlerts(repositoryHref: string) {
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

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      main_commit AS (
        SELECT repository_git_refs.target_commit_id AS id
        FROM repository_git_refs, target_repo
        WHERE repository_git_refs.repository_id = target_repo.id
          AND repository_git_refs.name IN ('main', 'refs/heads/main')
        ORDER BY CASE WHEN repository_git_refs.name = 'refs/heads/main' THEN 0 ELSE 1 END
        LIMIT 1
      ),
      package_file AS (
        SELECT ${sqlLiteral(`{
  "dependencies": {
    "@playwright/test": "1.55.0"
  }
}
`)}::text AS content
      )
      INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
      SELECT target_repo.id,
             main_commit.id,
             'package.json',
             package_file.content,
             md5(package_file.content),
             length(package_file.content)
      FROM target_repo, main_commit, package_file
      ON CONFLICT (repository_id, commit_id, lower(path))
      DO UPDATE SET content = EXCLUDED.content,
                    oid = EXCLUDED.oid,
                    byte_size = EXCLUDED.byte_size;
      `,
  ]);
}

function disableDependabot(repositoryHref: string) {
  const databaseUrl = requireTestDatabase();
  const [, owner, repo] = repositoryHref.split("/");
  runPsql(databaseUrl, [
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
  ]);
}

test.setTimeout(90_000);

test("repository Dependabot alerts support list filters, triage writes, security updates, disabled state, and final screenshots", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  test.skip(
    skipWithoutTestDb(),
    "repository Dependabot smoke needs a database URL",
  );
  const seeded = await seed({ scenes: ["treeRefs", "dependencyGraph"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDependabotAlerts(repositoryHref);
  await signIn(page, seeded, "owner");

  await page.goto(`${repositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Dependabot alerts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Dependabot Dependency alerts and updates",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByText("Playwright test runner demo advisory").first(),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "@playwright/test" }).first(),
  ).toHaveAttribute("href", /\/security\/dependabot\/\d+/);
  await expect(
    page.getByRole("link", { name: "package.json" }).first(),
  ).toHaveAttribute("href", `${repositoryHref}/blob/main/package.json`);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-002-final-list"),
  });

  await page.getByRole("button", { name: "Package: All packages" }).click();
  const packageFilterLink = page
    .getByRole("menuitem", { name: /npm:@playwright\/test/ })
    .first();
  await expect(packageFilterLink).toHaveAttribute(
    "href",
    /package=npm%3A%40playwright%2Ftest/,
  );
  await packageFilterLink.click();
  await expect(page).toHaveURL(/package=npm%3A%40playwright%2Ftest/);
  await expect(
    page.getByText("Playwright test runner demo advisory").first(),
  ).toBeVisible();

  await page.getByRole("button", { name: "Select all visible" }).click();
  await expect(
    page.getByRole("button", { name: "Clear visible" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Dismiss selected" }),
  ).toBeVisible();

  await page
    .getByRole("link", { name: "Playwright test runner demo advisory" })
    .first()
    .click();
  await expect(
    page.getByRole("heading", { name: "Playwright test runner demo advisory" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "package.json" }).first(),
  ).toHaveAttribute("href", `${repositoryHref}/blob/main/package.json`);
  await page.getByLabel("Dismiss reason").selectOption("not_used");
  await page
    .getByLabel("Optional comment")
    .fill("Only a browser smoke fixture uses this.");
  await page.getByRole("button", { name: "Dismiss alert" }).click();
  await expect(page.getByText("Dismiss saved.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Reopen alert" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Reopen alert" }).click();
  await expect(page.getByText("Reopen saved.")).toBeVisible();
  await page.getByRole("button", { name: "Create security update PR" }).click();
  await expect(
    page.getByRole("link", { name: "Open security update PR" }),
  ).toHaveAttribute("href", /\/pull\/\d+/);
  await page.getByRole("checkbox", { name: /dash-/ }).check();
  await page.getByRole("button", { name: "Save assignments" }).click();
  await expect(page.getByText("Assignments saved.")).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Dependabot alert timeline" }),
  ).toContainText("Updated Dependabot alert assignees.");
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-002-phase3-alert-detail"),
  });

  await page.goto(`${repositoryHref}/security/dependabot`);
  await page.getByRole("button", { name: "Package: All packages" }).click();
  await page.getByRole("menuitem", { name: /npm:@playwright\/test/ }).click();
  await page.getByRole("button", { name: "Select all visible" }).click();
  await expect(
    page.getByRole("button", { name: "Clear visible" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Dismiss selected" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(
      testInfo,
      "code-security-002-phase4-bulk-security-update",
    ),
  });

  disableDependabot(repositoryHref);
  await page.goto(`${repositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Vulnerability alerts are disabled." }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open vulnerability settings" }),
  ).toHaveAttribute("href", `${repositoryHref}/settings/security`);

  seedDependabotAlerts(repositoryHref);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${repositoryHref}/security/dependabot`);
  await expect(
    page.getByRole("heading", { name: "Dependabot alerts" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-002-final-mobile"),
  });
});
