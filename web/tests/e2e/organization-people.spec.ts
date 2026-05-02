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

test.skip(!databaseUrl, "organization people E2E needs a test database");

test("organization people routes render members with URL-backed search and pagination", async ({
  page,
}) => {
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);

  await page.goto(`${seeded.organizationProfileHref}?tab=people`);
  await expect(page.getByRole("heading", { name: "People" })).toBeVisible();
  await expect(
    page.getByRole("complementary", { name: "Organization permissions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Dashboard Tester" }),
  ).toHaveAttribute("href", /\/dash-/);
  await expect(
    page.getByRole("link", { name: "Open Profile Action Viewer" }),
  ).toHaveAttribute("href", /\/profile-viewer-/);
  await expect(page.getByText("Owner", { exact: true })).toBeVisible();
  await expect(page.getByText("Member", { exact: true })).toBeVisible();

  await page.goto(`${seeded.organizationProfileHref}/people?pageSize=1`);
  await expect(page).toHaveURL(
    /\/orgs\/org-profile-[^/]+\/people\?pageSize=1$/,
  );
  await expect(page.getByText(/1-1 of 2/)).toBeVisible();
  await page.getByRole("link", { name: "Next" }).click();
  await expect(page).toHaveURL(/\/people\?page=2&pageSize=1$/);
  await expect(page.getByRole("link", { name: "Previous" })).toHaveAttribute(
    "href",
    /\/people\?pageSize=1$/,
  );

  await page.getByLabel("Search organization people").fill("Profile Action");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(/q=Profile\+Action/);
  await expect(
    page.getByRole("link", { name: "Open Profile Action Viewer" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Dashboard Tester" }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("link", { name: "Search: Profile Action x" }),
  ).toHaveAttribute("href", /\/people\?pageSize=1$/);
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase4-people.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole("heading", { name: "People" })).toBeVisible();
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase4-people-mobile.jpg",
  });
});
