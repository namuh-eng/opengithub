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
  "settings navigation E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("personal settings sidebar highlights sections and keeps routes concrete", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/settings/profile");
  await expect(
    page.getByRole("heading", { name: "Personal settings" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { exact: true, name: "Public profile" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Profile" })).toHaveAttribute(
    "aria-current",
    "page",
  );

  await page.getByRole("link", { name: "Emails" }).click();
  await expect(page).toHaveURL(/\/settings\/emails$/);
  await expect(page.getByRole("heading", { name: "Emails" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Emails" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/nav-001-phase3-personal-settings.jpg",
  });
});

test("repository settings sidebar preserves repository context", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings`);
  const settingsNav = page.getByRole("navigation", {
    name: "Repository settings navigation",
  });
  await expect(page.getByRole("heading", { name: "General" })).toBeVisible();
  await expect(
    settingsNav.getByRole("link", { name: "General" }),
  ).toHaveAttribute("aria-current", "page");
  await settingsNav.getByRole("link", { name: "Branches" }).click();
  await expect(page).toHaveURL(
    new RegExp(`${seeded.firstRepositoryHref}/settings/branches$`),
  );
  await expect(page.getByRole("heading", { name: "Branches" })).toBeVisible();
  await expect(
    settingsNav.getByRole("link", { name: "Branches" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.getByRole("link", { name: "Code" })).toHaveAttribute(
    "href",
    seeded.firstRepositoryHref,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/nav-001-phase3-settings-shell.jpg",
  });
});
