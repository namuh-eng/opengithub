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

function seedDependencies(repositoryHref: string) {
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
      target_ref AS (
        SELECT repository_git_refs.target_commit_id AS commit_id
        FROM repository_git_refs
        JOIN target_repo ON target_repo.id = repository_git_refs.repository_id
        WHERE repository_git_refs.name = 'refs/heads/main'
        LIMIT 1
      ),
      files(path, content, oid) AS (
        VALUES
          (
            'package.json',
            '{"dependencies":{"@playwright/test":"^1.56.0"},"devDependencies":{"vitest":"^4.0.0"}}',
            'dependency-package-${suffix}'
          ),
          (
            'package-lock.json',
            '{"packages":{"node_modules/@playwright/test":{"version":"1.56.0"},"node_modules/vitest":{"version":"4.0.0"}}}',
            'dependency-package-lock-${suffix}'
          ),
          (
            'crates/api/Cargo.toml',
            '[package]\\nname = "opengithub-api"\\n[dependencies]\\nsqlx = "0.8"',
            'dependency-cargo-${suffix}'
          )
      )
      INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
      SELECT target_repo.id, target_ref.commit_id, files.path, files.content, files.oid, length(files.content)
      FROM target_repo, target_ref, files
      ON CONFLICT (repository_id, commit_id, lower(path))
      DO UPDATE SET content = EXCLUDED.content, oid = EXCLUDED.oid, byte_size = EXCLUDED.byte_size;
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

test.skip(
  !databaseUrl,
  "repository Dependency graph smoke needs a database URL",
);
test.setTimeout(90_000);

test("repository Dependencies renders filters, rows, and concrete actions", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedDependencies(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/network/dependencies`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Dependencies" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Dependency graph Dependencies and dependents",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { exact: true, name: "Dependencies" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { exact: true, name: "Dependents" }),
  ).toHaveAttribute("href", `${seeded.treeRepositoryHref}/network/dependents`);
  await expect(page.getByLabel("Dependency summary metrics")).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Repository dependencies list" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Search" }).fill("playwright");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page).toHaveURL(/q=playwright/);
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Ecosystem: All ecosystems" }).click();
  await page.getByRole("menuitem", { name: /npm/ }).click();
  await expect(page).toHaveURL(/ecosystem=npm/);
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Direct" }).click();
  await expect(page).toHaveURL(/relationship=direct/);
  await expect(
    page.getByRole("list", { name: "Indexed dependency manifests" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Export SBOM" }).click();
  await expect(page.getByText("Latest SBOM ready")).toBeVisible();
  const downloadHref = await page
    .getByRole("link", { name: "Download SBOM" })
    .getAttribute("href");
  expect(downloadHref).toMatch(/\/network\/dependencies\/sbom\/.+/);
  const sbom = await page.request.get(downloadHref ?? "");
  expect(sbom.status()).toBe(200);
  expect(sbom.headers()["content-type"]).toContain("json");
  expect(sbom.headers()["content-disposition"]).toContain("attachment");
  const sbomBody = await sbom.json();
  expect(sbomBody.spdxVersion).toBe("SPDX-2.3");
  expect(JSON.stringify(sbomBody)).toContain("@playwright/test");
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-005-phase3-sbom-export.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { exact: true, name: "Dependencies" }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
});
