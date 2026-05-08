import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
  profileActionCookieValue: string;
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

function setPagesDisplayState(
  firstRepositoryHref: string,
  updates: {
    artifact?: boolean;
    certificateStatus?: string;
    dnsStatus?: string;
    domain?: string;
    httpsEnforced?: boolean;
    provisioningStatus?: string;
  },
) {
  const { owner, repo } = repositoryParts(firstRepositoryHref);
  const assignments = [
    updates.domain === undefined
      ? null
      : `custom_domain = ${updates.domain ? sqlLiteral(updates.domain) : "NULL"}`,
    updates.domain === undefined
      ? null
      : `dns_challenge_name = ${updates.domain ? sqlLiteral(`_opengithub-pages.${updates.domain}`) : "NULL"}`,
    updates.domain === undefined
      ? null
      : `dns_challenge_value = ${updates.domain ? sqlLiteral(`og-pages-${repo}`) : "NULL"}`,
    updates.dnsStatus ? `dns_status = ${sqlLiteral(updates.dnsStatus)}` : null,
    updates.certificateStatus
      ? `certificate_status = ${sqlLiteral(updates.certificateStatus)}`
      : null,
    updates.provisioningStatus
      ? `provisioning_status = ${sqlLiteral(updates.provisioningStatus)}`
      : null,
    updates.httpsEnforced === undefined
      ? null
      : `https_enforced = ${updates.httpsEnforced ? "true" : "false"}`,
    "updated_at = now()",
  ].filter(Boolean);

  runSql(`
    WITH repo AS (
      SELECT repositories.id
      FROM repositories
      JOIN users ON users.id = repositories.owner_user_id
      WHERE users.username = ${sqlLiteral(owner)}
        AND repositories.name = ${sqlLiteral(repo)}
    )
    UPDATE pages_sites
    SET ${assignments.join(", ")}
    WHERE repository_id = (SELECT id FROM repo);
  `);

  if (updates.artifact) {
    runSql(`
      WITH repo AS (
        SELECT repositories.id
        FROM repositories
        JOIN users ON users.id = repositories.owner_user_id
        WHERE users.username = ${sqlLiteral(owner)}
          AND repositories.name = ${sqlLiteral(repo)}
      ),
      latest AS (
        SELECT pages_deployments.id
        FROM pages_deployments
        WHERE repository_id = (SELECT id FROM repo)
        ORDER BY created_at DESC
        LIMIT 1
      )
      UPDATE pages_deployments
      SET status = 'deployed',
          conclusion = 'success',
          artifact_storage_key = 'pages/e2e/final-live',
          artifact_manifest = '{"artifactCount":2,"storageMode":"local_metadata","totalBytes":256}'::jsonb,
          build_log_excerpt = 'Published 2 Pages artifact(s) to pages/e2e/final-live using local_metadata storage metadata.',
          completed_at = now()
      WHERE id = (SELECT id FROM latest);
    `);
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
        DASHBOARD_E2E_EMPTY: "0",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard, value?: string) {
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: value ?? seeded.cookieValue,
    },
  ]);
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
  "repository Pages smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can mutate Pages settings and forbidden users do not see private metadata", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const seeded = seedDashboard();
  const { repo: seededRepoName } = repositoryParts(seeded.firstRepositoryHref);
  const verifiedDomain = `verified-${seededRepoName}.example.com`;
  const brokenDomain = `broken-${seededRepoName}.example.com`;
  const liveDomain = `live-${seededRepoName}.example.com`;
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings/pages`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-disabled.jpg",
  });
  await expect(
    page.getByRole("heading", { name: /Pages$/ }).first(),
  ).toBeVisible();
  await expect(page.getByText("Publishing source")).toBeVisible();
  await page.getByLabel("Source", { exact: true }).selectOption("actions");
  const workflowOption = await page
    .getByLabel("Actions workflow", { exact: true })
    .locator("option")
    .evaluateAll((options) =>
      options
        .map((option) => option.getAttribute("value") ?? "")
        .find((value) => value.length > 0),
    );
  if (workflowOption) {
    await page
      .getByLabel("Actions workflow", { exact: true })
      .selectOption(workflowOption);
    await page.getByLabel("Artifact name", { exact: true }).fill("public");
    await page.getByRole("button", { name: "Save source" }).click();
    await expect(page.getByText("Actions source saved.")).toBeVisible();
    await expect(page.getByText("GitHub Actions · public")).toBeVisible();
  } else {
    await expect(
      page.getByLabel("Actions workflow", { exact: true }),
    ).toBeEnabled();
    await expect(page.getByLabel("Artifact name", { exact: true })).toHaveValue(
      "github-pages",
    );
  }
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-actions-source.jpg",
  });

  await page.getByLabel("Source", { exact: true }).selectOption("branch");
  const branchOption = await page
    .getByLabel("Branch", { exact: true })
    .locator("option")
    .evaluateAll((options) =>
      options
        .map((option) => option.getAttribute("value") ?? "")
        .find((value) => value.length > 0),
    );
  expect(branchOption).toBeTruthy();
  await page
    .getByLabel("Branch", { exact: true })
    .selectOption(branchOption ?? "");
  await page.getByLabel("Folder", { exact: true }).selectOption("/");
  await page.getByRole("button", { name: "Save source" }).click();
  await expect(
    page.getByText("Branch source saved and a Pages deployment was queued."),
  ).toBeVisible({ timeout: 15_000 });
  await expect(
    page.getByText(`${branchOption} · /(root)`).first(),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-branch-source.jpg",
  });
  await expect(
    page.getByRole("button", { name: "Deploy saved source" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Deploy saved source" }).click();
  await expect(
    page.getByText("Pages deployment queued from the saved branch source."),
  ).toBeVisible();
  await expect(page.getByText("Domain and HTTPS")).toBeVisible();
  await page
    .getByLabel("Domain", { exact: true })
    .fill(`docs-${Date.now()}.example.com`);
  await page.getByRole("button", { name: "Save domain" }).click();
  await expect(page.getByText("Custom domain saved.")).toBeVisible();
  await expect(page.getByText("og-pages-", { exact: false })).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-domain-pending.jpg",
  });
  await page.getByRole("button", { name: "Recheck DNS" }).click();
  await expect(
    page.getByText("DNS verification rechecked from the Pages API."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enforce HTTPS" }),
  ).toBeDisabled();
  setPagesDisplayState(seeded.firstRepositoryHref, {
    certificateStatus: "issued",
    dnsStatus: "verified",
    domain: verifiedDomain,
    httpsEnforced: false,
    provisioningStatus: "ready",
  });
  await page.reload();
  await expect(page.getByText("verified").first()).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enforce HTTPS" }),
  ).toBeEnabled();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-domain-verified.jpg",
  });

  setPagesDisplayState(seeded.firstRepositoryHref, {
    certificateStatus: "failed",
    dnsStatus: "misconfigured",
    domain: brokenDomain,
    httpsEnforced: false,
    provisioningStatus: "failed",
  });
  await page.reload();
  await expect(page.getByText("misconfigured").first()).toBeVisible();
  await expect(page.getByText("Certificate: failed")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-domain-error.jpg",
  });

  setPagesDisplayState(seeded.firstRepositoryHref, {
    artifact: true,
    certificateStatus: "issued",
    dnsStatus: "verified",
    domain: liveDomain,
    httpsEnforced: true,
    provisioningStatus: "ready",
  });
  await page.reload();
  await expect(page.getByText("Live", { exact: true })).toBeVisible();
  await expect(page.getByText(/Published 2 Pages artifact/)).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-live-deployment.jpg",
  });

  await page.getByRole("button", { name: "Unpublish Pages" }).click();
  await page.getByRole("button", { name: "Confirm unpublish" }).click();
  await expect(
    page.getByText("Pages unpublished. Repository files were preserved."),
  ).toBeVisible();
  await expect(page.getByText("Recent activity")).toBeVisible();
  await expect(
    page
      .locator("section")
      .filter({ hasText: "Recent activity" })
      .getByRole("link", { name: "Actions" }),
  ).toHaveAttribute("href", `${seeded.firstRepositoryHref}/actions`);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-unpublished.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    )
    .toBe(true);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-mobile.jpg",
  });

  await page.context().clearCookies();
  await signIn(page, seeded, seeded.profileActionCookieValue);
  await page.goto(`${seeded.firstRepositoryHref}/settings/pages`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
  ).toBeVisible();
  await expect(page.getByText("og-pages-")).toHaveCount(0);
  await expect(page.getByText("cloudfront", { exact: false })).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-final-pages-forbidden.jpg",
  });
});
