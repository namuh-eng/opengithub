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
    page.getByRole("link", { name: "Website namuh.co" }),
  ).toHaveAttribute("href", "https://namuh.co");
  await expect(page.getByRole("button", { name: "Sponsor" })).toBeDisabled();
  await expect(
    page.getByRole("navigation", { name: "Organization sections" }),
  ).toBeVisible();
  await page.getByRole("link", { name: /Repositories \d+/ }).click();
  await expect(page).toHaveURL(/\/orgs\/org-profile-[^?]+\?tab=repositories$/);
  await expect(
    page.getByRole("heading", { name: /Repositories for/ }),
  ).toBeVisible();

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
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-001-phase3-repository-preview.jpg",
  });
});
