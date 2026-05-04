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

function slugFromProfileHref(href: string) {
  const parts = href.split("/").filter(Boolean);
  const organizationsIndex = parts.indexOf("organizations");
  if (organizationsIndex >= 0 && parts[organizationsIndex + 1]) {
    return parts[organizationsIndex + 1];
  }
  const orgsIndex = parts.indexOf("orgs");
  if (orgsIndex >= 0 && parts[orgsIndex + 1]) {
    return parts[orgsIndex + 1];
  }
  throw new Error(`seeded href did not include a slug: ${href}`);
}

test.skip(
  !databaseUrl,
  "organization member privileges E2E needs a test database",
);
test.setTimeout(60_000);

test("owner manages organization member privileges", async ({ page }) => {
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);
  const slug = slugFromProfileHref(seeded.organizationProfileHref);

  await page.goto(`/organizations/${slug}/settings/member_privileges`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Member Privileges" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Member privileges" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("heading", { name: "Base permissions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Creation visibility" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Team creation" }),
  ).toBeVisible();

  await page.getByLabel("Public repositories").uncheck();
  await page.getByRole("button", { name: "Save repository creation" }).click();
  await expect(
    page.getByText("Repository creation policy updated"),
  ).toBeVisible();

  await page.getByLabel("Members can create teams").uncheck();
  await page.getByRole("button", { name: "Save team creation" }).click();
  await expect(page.getByText("Team creation policy updated")).toBeVisible();

  await page.getByLabel("None").nth(0).check();
  await page.getByRole("button", { name: "Save base permission" }).click();
  await expect(
    page.getByRole("dialog", { name: "Confirm organization policy change" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Confirm and save" }).click();
  await expect(
    page.getByText("Base repository permission updated"),
  ).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-005-phase2-member-privileges.jpg",
  });

  await page.setViewportSize({ width: 390, height: 900 });
  const bodyWidths = await page.locator("body").evaluate((body) => ({
    clientWidth: body.clientWidth,
    scrollWidth: body.scrollWidth,
  }));
  expect(bodyWidths.scrollWidth).toBeLessThanOrEqual(bodyWidths.clientWidth);
});
