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

test.skip(!databaseUrl, "organization profile E2E needs a test database");

test("organization overview renders API data and concrete header controls", async ({
  page,
}) => {
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);

  await page.goto(seeded.organizationProfileHref);
  await expect(
    page.getByRole("heading", { name: "Namuh Engineering" }),
  ).toBeVisible();
  await expect(page.getByText("@org-profile-")).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "Verified" }),
  ).toHaveAttribute("href", "https://namuh.co");
  await expect(
    page.getByRole("link", { exact: true, name: "Verified" }),
  ).toHaveAttribute("title", "Verified domain namuh.co");
  await expect(
    page.getByRole("link", { name: "Website namuh.co" }),
  ).toHaveAttribute("href", "https://namuh.co");
  await expect(
    page.getByRole("button", { exact: true, name: "Sponsor" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("navigation", { name: "Organization sections" }),
  ).toBeVisible();
  await page.getByRole("link", { name: /Repositories \d+/ }).click();
  await expect(page).toHaveURL(/\/orgs\/org-profile-[^?]+\?tab=repositories$/);
  await expect(
    page.getByRole("heading", { name: "Repositories" }),
  ).toBeVisible();
  await expect(
    page.getByLabel("Search organization repositories"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /opengithub-/ }).first(),
  ).toHaveAttribute("href", /\/org-profile-.+\/opengithub-/);
  await expect(
    page.getByRole("group", { name: "Display density" }),
  ).toBeVisible();

  await page.goto(`${seeded.organizationProfileHref}/repositories`);
  await expect(page).toHaveURL(/\/orgs\/org-profile-[^/]+\/repositories$/);
  await expect(
    page.getByRole("heading", { name: "Repositories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /opengithub-/ }).first(),
  ).toBeVisible();
  await page.goto(`${seeded.organizationProfileHref}/repositories?pageSize=1`);
  await expect(page.getByText(/1-1 of \d+/)).toBeVisible();
  await page.getByRole("link", { name: "Next" }).click();
  await expect(page).toHaveURL(/\/repositories\?page=2&pageSize=1$/);
  await expect(page.getByRole("link", { name: "Previous" })).toHaveAttribute(
    "href",
    /\/repositories\?pageSize=1$/,
  );
  await page.getByRole("link", { name: "Compact density" }).click();
  await expect(page).toHaveURL(/density=compact/);
  await expect(page).toHaveURL(/page=2/);
  await expect(page).toHaveURL(/pageSize=1/);
  await page.getByLabel("Language").selectOption("TypeScript");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(/language=TypeScript/);
  await expect(page).toHaveURL(/density=compact/);
  await expect(page).toHaveURL(/pageSize=1/);
  await expect(
    page.getByRole("link", { name: /ralph-/ }).first(),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: /opengithub-/ })).toHaveCount(0);
  await page.getByRole("link", { name: /All \d+/ }).click();
  await expect(page).toHaveURL(/\/repositories\?/);
  await expect(page).not.toHaveURL(/type=/);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Repositories" }),
  ).toBeVisible();
  const repositoryOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(repositoryOverflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase3-repositories-mobile.jpg",
  });
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase2-repositories-shell.jpg",
  });

  await page.goto(seeded.organizationProfileHref);
  const repositoryLink = page
    .getByRole("link", {
      name: /Open org-profile-[^/]+\/opengithub-/,
    })
    .first();
  await expect(repositoryLink).toHaveAttribute(
    "href",
    /\/org-profile-.+\/opengithub-/,
  );
  const pinnedRegion = page.getByRole("region", {
    name: "Pinned repositories",
  });
  await expect(pinnedRegion.getByText("1 stars")).toBeVisible();
  await expect(pinnedRegion.getByText("1 open issues")).toBeVisible();
  await expect(pinnedRegion.getByText("1 open pull requests")).toBeVisible();
  await expect(pinnedRegion.getByText("MIT License")).toBeVisible();
  await expect(pinnedRegion.getByText("Template")).toBeVisible();
  await expect(pinnedRegion.getByText("developer-tools")).toBeVisible();
  await repositoryLink.click();
  await expect(page).toHaveURL(/\/org-profile-.+\/opengithub-/);

  await page.goto(seeded.organizationProfileHref);
  const previewLink = page
    .getByRole("link", {
      name: /Open org-profile-[^/]+\/ralph-/,
    })
    .first();
  await expect(previewLink).toHaveAttribute("href", /\/org-profile-.+\/ralph-/);
  await expect(page.getByRole("link", { name: "View people" })).toHaveAttribute(
    "href",
    /\/orgs\/org-profile-.+\?tab=people/,
  );
  await expect(
    page.getByText("2 visible people including private members."),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Open Dashboard Tester/ }),
  ).toHaveAttribute("href", /\/dash-/);
  await expect(
    page.getByLabel("Rust 75% of visible organization code"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "developer-tools, 1 repositories",
    }),
  ).toHaveAttribute(
    "href",
    /\/orgs\/org-profile-.+\/repositories\?q=topic%3Adeveloper-tools/,
  );
  await expect(
    page.getByRole("button", { name: "Sponsor preview unavailable" }),
  ).toBeDisabled();
  await page.keyboard.press("Tab");
  await expect(page.locator(":focus")).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-001-phase3-repository-preview.jpg",
  });
  await page.goto(seeded.organizationProfileHref);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-001-final-desktop.jpg",
  });
});

test("organization secondary panels fit on mobile without dead controls", async ({
  page,
}) => {
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(seeded.organizationProfileHref);

  await expect(
    page.getByRole("heading", { name: "Namuh Engineering" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Open Dashboard Tester/ }),
  ).toBeVisible();
  await expect(
    page.getByLabel("Rust 75% of visible organization code"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "developer-tools, 1 repositories",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Sponsor preview unavailable" }),
  ).toBeDisabled();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-001-phase4-mobile.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-001-final-mobile.jpg",
  });
});
