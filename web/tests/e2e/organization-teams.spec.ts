import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededOrganizationTeams = {
  cookieName: string;
  cookieValue: string;
  profileActionCookieValue: string;
  organizationProfileHref: string;
  organizationEmptyTeamsHref: string;
};

function seedOrganizationTeams(): SeededOrganizationTeams {
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
  return JSON.parse(output) as SeededOrganizationTeams;
}

async function signIn(
  page: Page,
  seeded: SeededOrganizationTeams,
  cookieValue = seeded.cookieValue,
) {
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: cookieValue,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      sameSite: "Lax",
      secure: false,
    },
  ]);
}

test.skip(!databaseUrl, "organization teams E2E needs a test database");

test("organization teams directory supports owner/member views, filters, and navigation", async ({
  page,
}) => {
  test.setTimeout(60_000);
  const seeded = seedOrganizationTeams();
  await signIn(page, seeded);

  await page.goto(`${seeded.organizationProfileHref}/teams`);
  await expect(page.getByRole("heading", { name: "Teams" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Open Platform Maintainers/ }),
  ).toHaveAttribute("href", /\/orgs\/org-profile-[^/]+\/teams\/platform-/);
  await expect(page.getByText("Security Response")).toBeVisible();
  await expect(page.getByRole("link", { name: "Secret" })).toBeVisible();
  await expect(page.getByText("Mention notifications").first()).toBeVisible();

  await page.getByLabel("Search organization teams").fill("frontend");
  await page.getByLabel("Filter team visibility").selectOption("visible");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(
    /\/orgs\/org-profile-[^/]+\/teams\?q=frontend&visibility=visible$/,
  );
  await expect(page.getByText("Frontend Studio")).toBeVisible();
  await expect(page.getByText("Parent")).toBeVisible();
  await expect(page.getByText("Security Response")).toHaveCount(0);

  const teamHref = await page
    .getByRole("link", { name: /Open Frontend Studio/ })
    .getAttribute("href");
  expect(teamHref).toBeTruthy();
  await page.goto(teamHref ?? "");
  await expect(
    page.getByRole("heading", { name: /org-profile-/ }),
  ).toBeVisible();
  await expect(page.getByText(/will show members, repositories/)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 850 });
  await page.goto(`${seeded.organizationProfileHref}/teams`);
  const scrollWidth = await page.evaluate(() => document.body.scrollWidth);
  expect(scrollWidth).toBeLessThanOrEqual(390);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-004-phase2-teams-directory.jpg",
  });

  await signIn(page, seeded, seeded.profileActionCookieValue);
  await page.goto(`${seeded.organizationProfileHref}/teams`);
  await expect(page.getByText("Platform Maintainers")).toBeVisible();
  await expect(page.getByText("Frontend Studio")).toBeVisible();
  await expect(page.getByText("Security Response")).toHaveCount(0);
  await expect(page.getByRole("link", { name: /Secret/ })).toHaveCount(0);

  await signIn(page, seeded);
  await page.goto(seeded.organizationEmptyTeamsHref);
  await expect(page.getByText("Organize people by team")).toBeVisible();
  await expect(page.getByText("Flexible repository access")).toBeVisible();
  await expect(page.getByText("Request-to-join teams")).toBeVisible();
  await expect(page.getByText("Team mentions")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New team" }).first(),
  ).toHaveAttribute("href", /\/orgs\/org-empty-teams-[^/]+\/teams\/new$/);
  await expect(page.getByRole("link", { name: "Learn more" })).toHaveAttribute(
    "href",
    "/docs/api#organization-teams",
  );
});
