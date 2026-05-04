import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

function seedSession(): SeededSession {
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
        DASHBOARD_E2E_EMPTY: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededSession;
}

async function signIn(page: Page, seeded: SeededSession) {
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: seeded.cookieValue,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      sameSite: "Lax",
      secure: false,
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
  "profile, organization, and search navigation E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("profile, organization, team, and search skeleton routes stay navigable", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto("/mona?tab=repositories");
  await expect(page.getByRole("heading", { name: "mona" })).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Profile sections" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Stars" }).click();
  await expect(page).toHaveURL(/\/mona\?tab=stars$/);
  await expect(page.getByText(/Stars for mona/)).toBeVisible();
  await expectNoDeadControls(page);

  await page.goto("/orgs/namuh?tab=people");
  await expect(page.getByRole("heading", { name: "namuh" })).toBeVisible();
  await page.getByRole("link", { name: "Teams" }).click();
  await expect(page).toHaveURL(/\/orgs\/namuh\?tab=teams$/);
  await expect(page.getByText(/Teams for namuh/)).toBeVisible();
  await expectNoDeadControls(page);

  await page.goto("/orgs/namuh/teams/platform");
  await expect(
    page.getByRole("heading", { name: "namuh / platform" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "All teams" })).toHaveAttribute(
    "href",
    "/orgs/namuh/teams",
  );

  await page.goto("/search?q=router&type=code");
  await expect(
    page.getByRole("heading", { name: "Search indexed code" }),
  ).toBeVisible();
  await expect(page.getByText(/code results/)).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.getByRole("link", { name: "Users" }).click();
  await expect(page).toHaveURL(/\/search\?q=router&type=users$/);
  await expect(page.getByText("0 users results")).toBeVisible();
  await expectNoDeadControls(page);

  await page.goto("/organizations/new");
  await expect(
    page.getByRole("heading", { name: "Create a new organization" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/nav-001-phase4-profile-org-search.jpg",
  });
});
