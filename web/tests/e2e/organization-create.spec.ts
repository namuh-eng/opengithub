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

async function expectNoHorizontalOverflow(page: Page) {
  const metrics = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth);
}

test.skip(
  !databaseUrl,
  "organization create E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in organization plan picker opens setup and validates slugs", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto("/organizations/new");
  await expectNoDeadControls(page);
  await expect(
    page.getByRole("heading", { name: "Create a new organization" }),
  ).toBeVisible();
  await expect(page.getByLabel("Free plan")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Team plan unavailable" }),
  ).toBeDisabled();

  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Tell us about your organization" }),
  ).toBeVisible();

  const uniqueName = `Phase Two Org ${Date.now().toString(36)}`;
  const normalized = uniqueName.toLowerCase().replaceAll(/\s+/g, "-");
  await page.getByLabel("Organization name *").fill(uniqueName);
  await expect(
    page.getByText(`opengithub.namuh.co/${normalized}`),
  ).toBeVisible();
  await expect(page.getByText(`${normalized} is available.`)).toBeVisible();

  await page.getByLabel("Organization name *").fill("settings");
  await expect(
    page.getByText(/reserved|already taken|not available/i),
  ).toBeVisible();

  await page.getByLabel("Business or institution").check();
  await expect(page.getByLabel("Company name *")).toBeVisible();
  await page.getByLabel("Contact email *").fill("admin@example.com");
  await page
    .getByLabel("I accept the organization terms for this Free plan.")
    .check();
  await expect(
    page.getByRole("button", { name: "Create organization" }),
  ).toBeDisabled();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-001-phase2-plan-picker.jpg",
  });
});

test("organization create setup stays usable on mobile", async ({ page }) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/organizations/new");
  await expectNoHorizontalOverflow(page);
  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();
  await page.getByLabel("Organization name *").fill("Mobile Org!!");
  await expect(page.getByText("opengithub.namuh.co/mobile-org")).toBeVisible();
  await expectNoHorizontalOverflow(page);
});
