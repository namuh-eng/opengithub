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
        DASHBOARD_E2E_TREE_REFS: "1",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedTraffic(repositoryHref: string) {
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
      INSERT INTO repository_traffic_daily (
        repository_id, traffic_date, clones_total, clones_unique, visitors_total, visitors_unique
      )
      SELECT id, current_date - interval '2 days', 8, 3, 32, 14 FROM target_repo
      UNION ALL
      SELECT id, current_date - interval '1 day', 12, 5, 48, 20 FROM target_repo
      UNION ALL
      SELECT id, current_date, 14, 6, 55, 23 FROM target_repo
      ON CONFLICT (repository_id, traffic_date)
      DO UPDATE SET
        clones_total = EXCLUDED.clones_total,
        clones_unique = EXCLUDED.clones_unique,
        visitors_total = EXCLUDED.visitors_total,
        visitors_unique = EXCLUDED.visitors_unique;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
          AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))}
        LIMIT 1
      )
      INSERT INTO repository_referrers_daily (
        repository_id, traffic_date, referrer, total_views, unique_visitors
      )
      SELECT id, current_date - interval '1 day', 'https://search.opengithub.local/results?q=traffic', 24, 10 FROM target_repo
      UNION ALL
      SELECT id, current_date - interval '1 day', 'https://example.com/docs', 12, 6 FROM target_repo
      UNION ALL
      SELECT id, current_date - interval '1 day', 'https://very-long-referrer.example.com/docs/product/analytics/traffic/reports/2026/05/that-keeps-wrapping-in-the-table', 5, 2 FROM target_repo;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
          AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))}
        LIMIT 1
      )
      INSERT INTO repository_popular_content_daily (
        repository_id, traffic_date, path, title, total_views, unique_visitors
      )
      SELECT id, current_date - interval '1 day', 'README.md', 'README', 30, 12 FROM target_repo
      UNION ALL
      SELECT id, current_date - interval '1 day', 'src/main.rs', 'Application entrypoint', 16, 7 FROM target_repo
      UNION ALL
      SELECT id, current_date - interval '1 day', 'docs/product/analytics/traffic/reports/2026/05/very-long-file-name.md', 'Very long traffic report', 5, 2 FROM target_repo;
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

test.skip(!databaseUrl, "repository Traffic smoke needs a database URL");
test.setTimeout(90_000);

test("repository Traffic renders traffic analytics and concrete links", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedTraffic(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/graphs/traffic`);
  await expect(
    page.getByRole("heading", { name: "Traffic analytics" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Traffic Clone and visitor analytics",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.locator(".chip.active", { hasText: "Last 14 days" }),
  ).toBeVisible();
  await expect(page.locator(".chip", { hasText: "active days" })).toBeVisible();
  await expect(
    page.locator(".chip", { hasText: "Internal traffic excluded" }),
  ).toBeVisible();
  await expect(page.getByLabel("Traffic summary metrics")).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Clones line chart" }),
  ).toBeVisible();
  const clonePoint = page.getByRole("button", {
    name: /Clones .*12 clones, 5 unique cloners/,
  });
  await clonePoint.focus();
  await expect(page.getByText(/12 clones, 5 unique cloners/)).toBeVisible();
  const visitorPoint = page.getByRole("button", {
    name: /Visitors .*48 views, 20 unique visitors/,
  });
  await visitorPoint.hover();
  await expect(page.getByText(/48 views, 20 unique visitors/)).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Clones data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Visitors line chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Visitors data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);
  await expect(
    page.getByRole("link", {
      name: "https://search.opengithub.local/results?q=traffic",
    }),
  ).toHaveAttribute("rel", "noopener noreferrer");
  await expect(
    page.getByRole("link", { name: "Application entrypoint" }),
  ).toHaveAttribute("href", /\/blob\/.*src\/main\.rs$/);
  await expect(
    page.getByRole("link", {
      name: "https://very-long-referrer.example.com/docs/product/analytics/traffic/reports/2026/05/that-keeps-wrapping-in-the-table",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Very long traffic report" }),
  ).toHaveAttribute("href", /very-long-file-name\.md$/);

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-003-phase4-edge-cases.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Traffic analytics" }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
});
