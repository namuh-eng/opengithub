import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  treeRepositoryHref: string;
};

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
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
        DASHBOARD_E2E_TREE_REFS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

function seedSecurityAdvisories(repositoryHref: string) {
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
        SELECT repositories.id, users.id AS author_user_id, users.username AS author_login, users.avatar_url
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_security_advisories (
        repository_id, advisory_identifier, ghsa_id, cve_id, severity, status,
        title, summary, markdown_summary, markdown_details, package_ecosystem,
        package_name, affected_versions, patched_versions, vulnerable_range,
        cvss_vector, cvss_score, cvss_metrics, advisory_href, author_user_id,
        published_at, updated_at
      )
      SELECT target_repo.id,
             'GHSA-advisory-${suffix}-one',
             'GHSA-advisory-${suffix}-one',
             'CVE-2026-1234',
             'high',
             'published',
             'Token scope bypass in repository import workflow',
             'Repository imports could retain an overly broad token scope.',
             'Repository imports could retain an overly broad token scope.',
             'Patch the affected dependency before running imports.',
             'cargo',
             'opengithub-import',
             '< 1.2.3',
             '>= 1.2.3',
             '< 1.2.3',
             'CVSS:3.1/AV:N/AC:L/PR:L/UI:N/S:U/C:H/I:H/A:N',
             8.1,
             '{"attackVector":"network"}'::jsonb,
             ${sqlLiteral(`${repositoryHref}/security/advisories/GHSA-advisory-${suffix}-one`)},
             target_repo.author_user_id,
             now() - interval '1 hour',
             now() - interval '1 hour'
      FROM target_repo
      ON CONFLICT (repository_id, advisory_identifier)
      DO UPDATE SET title = EXCLUDED.title,
                    status = EXCLUDED.status,
                    severity = EXCLUDED.severity,
                    updated_at = now();

      WITH advisory AS (
        SELECT id FROM repository_security_advisories
        WHERE advisory_identifier = 'GHSA-advisory-${suffix}-one'
        LIMIT 1
      )
      INSERT INTO repository_security_advisory_cwes (advisory_id, cwe_id, name)
      SELECT advisory.id, 'CWE-284', 'Improper Access Control'
      FROM advisory
      ON CONFLICT (advisory_id, upper(cwe_id)) DO UPDATE
      SET name = EXCLUDED.name;

      WITH advisory AS (
        SELECT id FROM repository_security_advisories
        WHERE advisory_identifier = 'GHSA-advisory-${suffix}-one'
        LIMIT 1
      )
      INSERT INTO repository_security_advisory_credits (advisory_id, login, credit_type)
      SELECT advisory.id, 'security-reporter', 'reporter'
      FROM advisory
      ON CONFLICT (advisory_id, lower(login), credit_type) DO NOTHING;

      WITH advisory AS (
        SELECT id FROM repository_security_advisories
        WHERE advisory_identifier = 'GHSA-advisory-${suffix}-one'
        LIMIT 1
      )
      INSERT INTO repository_security_advisory_collaborators (advisory_id, login, role)
      SELECT advisory.id, 'jaeyun', 'author'
      FROM advisory
      ON CONFLICT (advisory_id, lower(login)) DO UPDATE
      SET role = EXCLUDED.role;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_security_advisories (
        repository_id, advisory_identifier, ghsa_id, severity, status, title,
        summary, package_ecosystem, package_name, vulnerable_range,
        affected_versions, patched_versions, advisory_href, published_at,
        updated_at
      )
      SELECT target_repo.id,
             'GHSA-advisory-${suffix}-two',
             'GHSA-advisory-${suffix}-two',
             'medium',
             'published',
             'Long advisory title wraps across the Editorial list without overflow',
             'Long package metadata stays readable on mobile and desktop.',
             'npm',
             'opengithub-advisory-authoring-with-long-name',
             '< 4.0.0',
             '< 4.0.0',
             '>= 4.0.0',
             ${sqlLiteral(`${repositoryHref}/security/advisories/GHSA-advisory-${suffix}-two`)},
             now() - interval '2 hours',
             now() - interval '2 hours'
      FROM target_repo
      ON CONFLICT (repository_id, advisory_identifier)
      DO UPDATE SET title = EXCLUDED.title,
                    status = EXCLUDED.status,
                    severity = EXCLUDED.severity,
                    updated_at = now();

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_security_advisories (
        repository_id, advisory_identifier, ghsa_id, severity, status, title,
        summary, package_name, vulnerable_range, advisory_href, updated_at
      )
      SELECT target_repo.id,
             'GHSA-advisory-${suffix}-draft',
             'GHSA-advisory-${suffix}-draft',
             'critical',
             'draft',
             'Private draft advisory',
             'This private draft should only be visible to maintainers.',
             'private-package',
             '< 9.9.9',
             ${sqlLiteral(`${repositoryHref}/security/advisories/GHSA-advisory-${suffix}-draft`)},
             now() - interval '30 minutes'
      FROM target_repo
      ON CONFLICT (repository_id, advisory_identifier)
      DO UPDATE SET title = EXCLUDED.title,
                    status = EXCLUDED.status,
                    updated_at = now();
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

test.skip(!databaseUrl, "repository advisory smoke needs a database URL");
test.setTimeout(90_000);

test("repository security advisories list filters, links, and mobile layout work", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedSecurityAdvisories(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/security/advisories`);
  await expect(
    page.getByRole("heading", { name: "Security advisories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Advisories Published security advisories",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { name: "New draft security advisory" }),
  ).toHaveAttribute(
    "href",
    `${seeded.treeRepositoryHref}/security/advisories/new`,
  );
  await expect(page.getByText("Token scope bypass")).toBeVisible();
  await expect(page.getByText("Private draft advisory")).not.toBeVisible();
  await expect(
    page.getByRole("link", { name: "View advisory" }).first(),
  ).toHaveAttribute("href", /\/security\/advisories\/GHSA-advisory-/);

  await page.getByRole("button", { name: /Severity:/ }).click();
  await expect(
    page.getByRole("menu", { name: "Severity options" }),
  ).toBeVisible();
  await page.getByRole("menuitem", { name: /High/ }).click();
  await expect(page).toHaveURL(/severity=high/);
  await expect(page.getByText("Token scope bypass")).toBeVisible();

  await page.getByRole("link", { name: /Draft 1/ }).click();
  await expect(page).toHaveURL(/state=draft/);
  await expect(page.getByText("Private draft advisory")).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/code-security-005-phase2-advisories-list.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.treeRepositoryHref}/security/advisories`);
  await expect(
    page.getByText("Long advisory title wraps across the Editorial list"),
  ).toBeVisible();
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
  await expectNoDeadControls(page);
});

test("repository security advisory detail renders and edits metadata", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedSecurityAdvisories(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/security/advisories`);
  await page.getByRole("link", { name: "View advisory" }).first().click();
  await expect(
    page.getByRole("heading", {
      name: "Token scope bypass in repository import workflow",
    }),
  ).toBeVisible();
  await expect(page.getByText("GHSA-advisory-")).toBeVisible();
  await expect(page.getByText("CVE-2026-1234")).toBeVisible();
  await expect(page.getByRole("button", { name: "Score 8.1" })).toBeVisible();
  await expect(page.getByText(/CWE-284 Improper Access Control/)).toBeVisible();

  await page
    .getByRole("textbox", { name: "Title" })
    .fill("Edited advisory title");
  await page
    .getByRole("combobox", { name: "Severity" })
    .selectOption("critical");
  await page.getByRole("button", { name: "Save advisory" }).click();
  await expect(page.getByText("Advisory metadata saved.")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Edited advisory title" }),
  ).toBeVisible();
  await expect(page.getByText("critical")).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/code-security-005-phase3-advisory-detail.jpg",
  });
});
