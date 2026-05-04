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

test.skip(!databaseUrl, "repository Pulse smoke needs a database URL");
test.setTimeout(90_000);

test("repository Pulse renders live overview data and concrete destinations", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/pulse`);
  await expect(
    page.getByRole("heading", { name: "Repository activity" }),
  ).toBeVisible();
  await expect(page.getByText("Insights").first()).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Pulse Activity summary for the selected period",
    }),
  ).toHaveAttribute("href", /\/pulse$/);
  await expect(
    page.getByRole("link", {
      name: "Contributors Contributor commit activity",
    }),
  ).toHaveAttribute("href", /\/graphs\/contributors$/);
  await page.getByRole("button", { name: "Period: Last week" }).click();
  await expect(page.getByRole("menu", { name: "Pulse period" })).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "Last 24 hours" }),
  ).toHaveAttribute("href", /\/pulse\?period=24h$/);
  await expect(
    page.getByRole("menuitem", { name: "Last month" }),
  ).toHaveAttribute("href", /\/pulse\?period=1m$/);
  await page.getByRole("menuitem", { name: "Last 3 days" }).click();
  await expect(page).toHaveURL(/\/pulse\?period=3d$/);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await expect(page.getByText(/May \d+, 2026 - May \d+, 2026/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);

  await expect(page.getByLabel("Pulse overview metrics")).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Merged pull requests/ }),
  ).toHaveAttribute("href", /\/pulls\?state=merged/);
  await expect(
    page.getByRole("link", { name: /Open pull requests/ }),
  ).toHaveAttribute("href", /\/pulls\?state=open/);
  await expect(
    page.getByRole("link", { name: /Closed issues/ }),
  ).toHaveAttribute("href", /\/issues\?state=closed/);
  await expect(page.getByRole("link", { name: /New issues/ })).toHaveAttribute(
    "href",
    /\/issues\?state=open.*sort=created-desc/,
  );
  await expect(
    page.getByRole("img", { name: "Top committers bar chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Top committers data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /\d+ commits/ }).first(),
  ).toHaveAttribute("href", /\/commits\/main\?.*until=/);
  await expect(
    page.getByRole("link", { name: /pulse-e2e|dashboard/ }).first(),
  ).toHaveAttribute("href", /^\/[^/]+$/);

  const releaseLink = page
    .getByRole("link", { name: /View releases|Release|preview/i })
    .first();
  await expect(releaseLink).toHaveAttribute("href", /\/releases/);
  const pullLink = page.locator('a[href*="/pull"]').first();
  await expect(pullLink).toHaveAttribute("href", /\/pulls|\/pull\//);
  const issueLink = page.locator('a[href*="/issues"]').first();
  await expect(issueLink).toHaveAttribute("href", /\/issues/);

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-001-final-desktop.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-001-final-mobile.jpg",
  });
});
