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

test("organization people routes render owner admin controls with URL-backed search and pagination", async ({
  page,
}) => {
  test.setTimeout(60_000);
  const seeded = seedOrganizationProfile();
  await signIn(page, seeded);

  await page.goto(`${seeded.organizationProfileHref}?tab=people`);
  await expect(
    page.getByRole("heading", { name: "People administration" }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "People administration tabs" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Dashboard Tester" }),
  ).toHaveAttribute("href", /\/dash-/);
  await expect(
    page.getByRole("link", { name: "Open Profile Action Viewer" }),
  ).toHaveAttribute("href", /\/profile-viewer-/);
  await expect(page.getByText("Owner", { exact: true })).toBeVisible();
  await expect(page.getByText("Member", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Bulk action" }),
  ).toBeDisabled();
  const ownerRow = page
    .locator("article")
    .filter({ hasText: "Dashboard Tester" });
  await ownerRow.getByRole("button", { name: "Row actions" }).click();
  await expect(
    page.getByText("Final owners cannot be demoted or removed."),
  ).toBeVisible();
  await expect(
    ownerRow.getByRole("button", { name: "Change role" }),
  ).toBeDisabled();
  await expect(
    ownerRow.getByRole("button", { name: "Remove from organization" }),
  ).toBeDisabled();
  await page.getByLabel("Select Profile Action Viewer").check();
  await expect(
    page.getByRole("button", { name: "Bulk action (1)" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Bulk action (1)" }).click();
  await expect(page.getByText(/Bulk membership mutations/)).toBeVisible();
  await page.getByRole("button", { name: "Export" }).click();
  await expect(page.getByRole("link", { name: "Export JSON" })).toHaveAttribute(
    "href",
    /\/api\/orgs\/org-profile-[^/]+\/people\/export\?format=json&tab=members/,
  );
  const jsonExportHref = await page
    .getByRole("link", { name: "Export JSON" })
    .getAttribute("href");
  expect(jsonExportHref).toBeTruthy();
  const jsonExport = await page.request.get(jsonExportHref ?? "");
  expect(jsonExport.status()).toBe(200);
  expect(jsonExport.headers()["content-type"]).toContain("application/json");
  const jsonExportBody = await jsonExport.text();
  expect(jsonExportBody).toContain("Dashboard Tester");
  expect(jsonExportBody).not.toContain("sha256:");
  await expect(page.getByRole("link", { name: "Export CSV" })).toHaveAttribute(
    "href",
    /\/api\/orgs\/org-profile-[^/]+\/people\/export\?format=csv&tab=members/,
  );
  const csvExportHref = await page
    .getByRole("link", { name: "Export CSV" })
    .getAttribute("href");
  expect(csvExportHref).toBeTruthy();
  const csvExport = await page.request.get(csvExportHref ?? "");
  expect(csvExport.status()).toBe(200);
  expect(csvExport.headers()["content-type"]).toContain("text/csv");
  expect(csvExport.headers()["content-disposition"]).toContain("attachment");
  const csvExportBody = await csvExport.text();
  expect(csvExportBody).toContain("login,display_name,role");
  expect(csvExportBody).toContain("Dashboard Tester");
  expect(csvExportBody).not.toContain("sha256:");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-export-menu.jpg",
  });
  await page.getByRole("button", { name: "Invite member" }).click();
  await expect(page.getByLabel("Invite member dialog")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-invite-dialog.jpg",
  });
  const inviteEmail = `phase3-${Date.now()}@opengithub.local`;
  await page.getByPlaceholder("member@example.com").fill(inviteEmail);
  await page.getByRole("combobox").selectOption("admin");
  await page.getByRole("button", { name: "Send invitation" }).click();
  await expect(
    page.getByText(/Invitation saved with degraded email delivery/),
  ).toBeVisible();
  await expect(page.getByText(inviteEmail).first()).toBeVisible();

  await page.getByRole("link", { name: /Failed invitations/ }).click();
  await expect(page.getByText("Email failed").first()).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-failed-invitations.jpg",
  });
  await page.getByRole("button", { name: "Retry" }).first().click();
  await expect(page.getByText(/Retried invitation/)).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-pending-invitations.jpg",
  });
  await page.getByRole("button", { name: "Cancel" }).first().click();
  await expect(page.getByText(/Canceled invitation/)).toBeVisible();

  await page.goto(`${seeded.organizationProfileHref}/people?pageSize=1`);
  await expect(page).toHaveURL(
    /\/orgs\/org-profile-[^/]+\/people\?pageSize=1$/,
  );
  await expect(page.getByText(/1-1 of 2/)).toBeVisible();
  const nextHref = await page
    .getByRole("link", { name: "Next" })
    .getAttribute("href");
  expect(nextHref).toBeTruthy();
  await page.goto(nextHref ?? "");
  await page.waitForURL(/\/people\?page=2&pageSize=1$/);
  await expect(page.getByText(/2-2 of 2/)).toBeVisible();
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
  await page.getByRole("button", { name: "Visibility: public" }).click();
  await page.getByRole("button", { name: "Private membership" }).click();
  await expect(page.getByText(/membership is now private/)).toBeVisible();
  const profileRow = page
    .locator("article")
    .filter({ hasText: "Profile Action Viewer" });
  await profileRow.getByRole("button", { name: "Row actions" }).click();
  await profileRow.getByRole("button", { name: "Change role" }).click();
  await page
    .getByLabel("Change role for Profile Action Viewer")
    .getByRole("combobox")
    .selectOption("admin");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-role-confirmation.jpg",
  });
  await page.getByRole("button", { name: "Confirm role change" }).click();
  await expect(page.getByText(/is now admin/)).toBeVisible();
  await profileRow
    .getByRole("button", { name: "Remove from organization" })
    .click();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-remove-confirmation.jpg",
  });
  await page.getByRole("button", { name: "Confirm removal" }).click();
  await expect(
    page.getByText(/was removed from the organization/),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Profile Action Viewer" }),
  ).toHaveCount(0);
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-phase3-invitations.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-phase4-member-actions.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-phase2-people-shell.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-members-table.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase4-people.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-final-people-desktop.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "People administration" }),
  ).toBeVisible();
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-phase4-people-mobile.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/orgs-002-final-people-mobile.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/org-admin-003-final-mobile.jpg",
  });
});
