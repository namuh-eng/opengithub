import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
  profileActionCookieValue: string;
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
  "repository settings access smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can load repository access settings shell", async ({ page }) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings/access`);

  await expect(
    page.getByRole("heading", { exact: true, name: "Access" }),
  ).toBeVisible();
  await expect(page.getByText(/Viewer: (Admin|Owner)/)).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "People with repository access" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Organization teams with access" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Pending invitations" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Repository role hierarchy" }),
  ).toBeVisible();

  const filter = page.getByLabel("Filter access");
  await filter.fill("dashboard");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page.getByRole("status")).toContainText("dashboard");
  await expect(page.getByLabel("Filter access")).toHaveValue("dashboard");
  await expect(page.getByRole("link", { name: "Clear" })).toHaveAttribute(
    "href",
    new RegExp(`${seeded.firstRepositoryHref}/settings/access$`),
  );

  await page.getByRole("link", { name: "Clear" }).click();
  await expect(
    page.getByRole("link", { name: "Add people" }).first(),
  ).toHaveAttribute("href", "#invite-people");
  await expect(
    page.getByRole("link", { name: "Add teams" }).first(),
  ).toHaveAttribute("href", "#invite-teams");
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-002-phase2-access-shell.jpg",
  });

  await page.context().clearCookies();
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: seeded.profileActionCookieValue,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      sameSite: "Lax",
      secure: false,
    },
  ]);
  await page.goto(`${seeded.firstRepositoryHref}/settings/access`);
  await expect(
    page.getByRole("heading", { name: "Repository access is restricted" }),
  ).toBeVisible();
});
