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
  await expect(
    page.getByRole("link", { name: /Period: Last week/ }),
  ).toHaveAttribute("href", /\/pulse$/);
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
    /\/issues\?/,
  );
  await expect(
    page.getByRole("img", { name: "Top committers bar chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Top committers data table" }),
  ).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/insights-001-phase2-pulse-overview.jpg",
  });
});
