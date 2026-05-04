import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededOrganizationProfile = {
  cookieName: string;
  cookieValue: string;
  organizationProfileHref: string;
};

function seedOrganizationProfile(): SeededOrganizationProfile {
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
        ORG_PROFILE_E2E: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededOrganizationProfile;
}

async function signIn(page: Page, seeded: SeededOrganizationProfile) {
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
  "organization settings profile E2E needs a test database",
);

test("owner opens organization profile settings shell", async ({ page }) => {
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);
  const slug = seeded.organizationProfileHref.split("/").filter(Boolean).at(-1);
  if (!slug) {
    throw new Error("seeded organization profile href did not include a slug");
  }

  await page.goto(`/organizations/${slug}/settings/profile`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Profile" }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", {
      name: "Organization settings navigation",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Personal settings" }),
  ).toHaveAttribute("href", "/settings/profile");
  await expect(
    page.getByRole("link", { name: "Organization settings" }),
  ).toHaveAttribute("href", `/organizations/${slug}/settings/profile`);
  await expect(
    page.getByRole("link", { name: "View organization" }),
  ).toHaveAttribute("href", `/orgs/${slug}`);
  await expect(page.getByLabel("Organization display name")).toBeVisible();
  await expect(page.getByLabel("Contact email")).toBeVisible();
  await expect(
    page.getByRole("textbox", { exact: true, name: "X" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Upload unavailable" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Save profile changes" }),
  ).toBeDisabled();
  await expect(page.getByText("Billing", { exact: true })).toHaveAttribute(
    "aria-disabled",
    "true",
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-002-phase2-settings-shell.jpg",
  });

  await page.goto(`/orgs/${slug}/settings`);
  await expect(page).toHaveURL(`/organizations/${slug}/settings/profile`);
});
