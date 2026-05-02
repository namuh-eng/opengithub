import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard) {
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
  "repository settings general smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can load repository general settings read surface", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings`);

  await expect(page.getByRole("heading", { name: "General" })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /\/alpha-[a-f0-9]+/ }),
  ).toBeVisible();
  await expect(page.getByLabel("Repository name")).toHaveValue(/alpha-/);
  await expect(page.getByText("Repository state")).toBeVisible();
  await expect(page.getByText("Feature toggles")).toBeVisible();
  await expect(page.getByText("Merge methods")).toBeVisible();
  await expect(page.getByText("Destructive actions")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "View branches" }),
  ).toHaveAttribute(
    "href",
    new RegExp(`${seeded.firstRepositoryHref}/branches$`),
  );
  await expect(
    page.getByRole("button", { name: "Delete repository unavailable" }),
  ).toBeDisabled();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-001-phase2-general-settings.jpg",
  });
});
