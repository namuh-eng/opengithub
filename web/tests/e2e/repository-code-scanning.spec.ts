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

function seedCodeScanningAlerts(repositoryHref: string) {
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
      main_ref AS (
        SELECT repository_git_refs.target_commit_id AS commit_id
        FROM repository_git_refs, target_repo
        WHERE repository_git_refs.repository_id = target_repo.id
          AND repository_git_refs.name IN ('main', 'refs/heads/main')
        ORDER BY CASE WHEN repository_git_refs.name = 'refs/heads/main' THEN 0 ELSE 1 END
        LIMIT 1
      ),
      settings AS (
        INSERT INTO repository_security_feature_settings (
          repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        SELECT target_repo.id,
               'code_scanning',
               'enabled',
               'Code scanning alerts are monitored.',
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
      run_one AS (
        INSERT INTO code_scanning_runs (
          repository_id, tool_name, tool_version, ref_name, commit_oid, status, completed_at
        )
        SELECT target_repo.id,
               'CodeQL',
               '2.17.0',
               'refs/heads/main',
               COALESCE(main_ref.commit_id::text, repeat('a', 40)),
               'completed',
               now() - interval '1 hour'
        FROM target_repo
        LEFT JOIN main_ref ON true
        RETURNING id, repository_id
      ),
      run_two AS (
        INSERT INTO code_scanning_runs (
          repository_id, tool_name, tool_version, ref_name, commit_oid, status, completed_at
        )
        SELECT target_repo.id,
               'Semgrep',
               NULL,
               'refs/heads/main',
               COALESCE(main_ref.commit_id::text, repeat('b', 40)),
               'completed',
               now() - interval '2 hours'
        FROM target_repo
        LEFT JOIN main_ref ON true
        RETURNING id, repository_id
      ),
      file_one AS (
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        SELECT target_repo.id,
               main_ref.commit_id,
               'crates/api/src/routes/search.rs',
               'fn query(input: String) { sqlx::query(&input); }',
               md5('search-${suffix}'),
               47
        FROM target_repo, main_ref
        ON CONFLICT (repository_id, commit_id, lower(path))
        DO UPDATE SET content = EXCLUDED.content,
                      oid = EXCLUDED.oid,
                      byte_size = EXCLUDED.byte_size
      ),
      file_two AS (
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        SELECT target_repo.id,
               main_ref.commit_id,
               'crates/api/src/domain/archive.rs',
               'fn unpack(path: String) { std::fs::read(path).unwrap(); }',
               md5('archive-${suffix}'),
               57
        FROM target_repo, main_ref
        ON CONFLICT (repository_id, commit_id, lower(path))
        DO UPDATE SET content = EXCLUDED.content,
                      oid = EXCLUDED.oid,
                      byte_size = EXCLUDED.byte_size
      ),
      alert_one AS (
        INSERT INTO code_scanning_alerts (
          repository_id, run_id, number, state, rule_id, rule_name, message, severity,
          security_severity, tool_name, path, start_line, end_line, ref_name, branch_name,
          fingerprint, code_snippet, rule_description, help_markdown
        )
        SELECT run_one.repository_id,
               run_one.id,
               1,
               'open',
               'rust/sql-injection',
               'Unsanitized SQL query',
               'User-controlled data reaches a SQL sink.',
               'warning',
               'high',
               'CodeQL',
               'crates/api/src/routes/search.rs',
               42,
               45,
               'refs/heads/main',
               'main',
               ${sqlLiteral(`codeql-${suffix}-1`)},
               'sqlx::query(&input)',
               'SQL queries should use bound parameters.',
               'Use query parameters instead of string-built SQL.'
        FROM run_one
        ON CONFLICT (repository_id, rule_id, path, start_line, fingerprint, ref_name)
        DO UPDATE SET updated_at = now()
        RETURNING id, repository_id
      )
      INSERT INTO code_scanning_alerts (
        repository_id, run_id, number, state, rule_id, rule_name, message, severity,
        security_severity, tool_name, path, start_line, end_line, ref_name, branch_name,
        fingerprint, code_snippet, rule_description, help_markdown
      )
      SELECT run_two.repository_id,
             run_two.id,
             2,
             'open',
             'rust/path-traversal',
             'Path traversal in archive reader',
             'Archive entries are joined without path normalization.',
             'error',
             'critical',
             'Semgrep',
             'crates/api/src/domain/archive.rs',
             88,
             NULL,
             'refs/heads/main',
             'main',
             ${sqlLiteral(`semgrep-${suffix}-2`)},
             'std::fs::read(path)',
             'Archive paths should be normalized before file access.',
             'Reject parent-directory traversals before reading files.'
      FROM run_two
      ON CONFLICT (repository_id, rule_id, path, start_line, fingerprint, ref_name)
      DO UPDATE SET updated_at = now();
      `,
    ],
    { stdio: "ignore" },
  );
}

function disableCodeScanning(repositoryHref: string) {
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
             'code_scanning',
             'disabled',
             'Code scanning is disabled for this repository.',
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

test.skip(!databaseUrl, "repository Code scanning smoke needs a database URL");
test.setTimeout(90_000);

test("repository Code scanning alerts support list filters, row navigation, disabled state, and screenshot evidence", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedCodeScanningAlerts(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/security/code-scanning`);
  await expect(
    page.getByRole("heading", { name: "Code scanning alerts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Code scanning Static analysis findings",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.getByText("Unsanitized SQL query").first()).toBeVisible();
  await expect(
    page.getByRole("link", { name: "crates/api/src/routes/search.rs:42" }),
  ).toHaveAttribute(
    "href",
    /\/blob\/refs%2Fheads%2Fmain\/crates\/api\/src\/routes\/search\.rs#L42/,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/code-security-003-phase2-alerts-list.jpg",
  });

  await page.getByRole("button", { name: "Tool: All tools" }).click();
  await page.getByRole("menuitem", { name: /CodeQL/ }).click();
  await expect(page).toHaveURL(/tool=CodeQL/);
  await expect(page.getByText("Unsanitized SQL query").first()).toBeVisible();

  await page.getByRole("button", { name: "Select all visible" }).click();
  await expect(
    page.getByRole("button", { name: "Clear visible" }),
  ).toBeVisible();
  await page
    .getByRole("link", { name: "Unsanitized SQL query" })
    .first()
    .click();
  await expect(page).toHaveURL(/\/security\/code-scanning\/1/);

  disableCodeScanning(seeded.treeRepositoryHref);
  await page.goto(`${seeded.treeRepositoryHref}/security/code-scanning`);
  await expect(
    page.getByRole("heading", { name: "Code scanning is not enabled." }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Enable code scanning" }),
  ).toHaveAttribute("href", `${seeded.treeRepositoryHref}/settings/security`);

  seedCodeScanningAlerts(seeded.treeRepositoryHref);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.treeRepositoryHref}/security/code-scanning`);
  await expect(
    page.getByRole("heading", { name: "Code scanning alerts" }),
  ).toBeVisible();
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
  await expectNoDeadControls(page);
});
