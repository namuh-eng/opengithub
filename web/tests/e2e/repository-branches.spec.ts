import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
  treeRepositoryHref: string;
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
        DASHBOARD_E2E_TREE_REFS: "1",
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

test.skip(!databaseUrl, "repository branches smoke needs a database URL");
test.setTimeout(90_000);

test("branches overview renders live rows and concrete actions", async ({
  page,
  context,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  await context.grantPermissions(["clipboard-write"], {
    origin: "http://localhost:3015",
  });

  await page.goto(`${seeded.treeRepositoryHref}/branches`);
  await expect(page.getByRole("heading", { name: "Branches" })).toBeVisible();
  await expect(page.getByText("Default branch", { exact: true })).toBeVisible();
  await expect(
    page.getByText("Active branches", { exact: true }),
  ).toBeVisible();

  const defaultSection = page.locator("section", { hasText: "Default branch" });
  await expect(
    defaultSection.getByRole("link", { name: "main" }),
  ).toBeVisible();
  await expect(
    defaultSection.getByRole("link", { name: "Commits" }),
  ).toHaveAttribute("href", /\/commits\/main/);

  await page.getByLabel("Search branches").fill("main");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/\/branches\?q=main/);
  await expect(page.getByRole("link", { name: "Clear" })).toBeVisible();

  await page.getByRole("tab", { name: /All/ }).click();
  await expect(page).toHaveURL(/\/branches\?tab=all/);
  await expect(page.getByText(/branches$/).first()).toBeVisible();

  await page.getByLabel("Search branches").fill("feature");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/\/branches\?tab=all&q=feature/);
  await expect(
    page.getByRole("link", { name: "Search: feature" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "feature/tree-nav" }),
  ).toHaveAttribute("href", /\/tree\/feature%2Ftree-nav$/);

  await page.goto(`${seeded.treeRepositoryHref}/branches?tab=all&pageSize=1`);
  await expect(page.getByRole("link", { name: "Next" })).toHaveAttribute(
    "href",
    /page=2&pageSize=1/,
  );
  await page.getByRole("link", { name: "Next" }).press("Enter");
  await expect(page).toHaveURL(/\/branches\?tab=all&page=2&pageSize=1$/);
  await expect(page.getByRole("link", { name: "Previous" })).toHaveAttribute(
    "href",
    /\/branches\?tab=all&pageSize=1$/,
  );

  await page.getByRole("tab", { name: /Stale/ }).click();
  await expect(page).toHaveURL(/\/branches\?tab=stale&pageSize=1$/);
  await expect(
    page.getByRole("link", { name: "release/old-tree" }),
  ).toBeVisible();
  const staleBranchRow = page
    .getByRole("link", { name: "release/old-tree" })
    .locator("xpath=ancestor::article[1]");
  await staleBranchRow.getByRole("link", { name: "Activity" }).click();
  await expect(page).toHaveURL(/\/branches\/release%2Fold-tree$/);
  await expect(
    page.getByRole("heading", { name: "release/old-tree" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Branches" })).toHaveAttribute(
    "href",
    /\/branches$/,
  );
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/release%2Fold-tree$/);
  await expect(page.getByRole("link", { name: "View rules" })).toHaveAttribute(
    "href",
    /\/settings\/branches\?branch=release%2Fold-tree/,
  );
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/branches-001-final-activity.jpg",
  });
  await page.goto(`${seeded.treeRepositoryHref}/branches?tab=stale`);

  await page
    .getByRole("button", { name: /Copy branch name/ })
    .first()
    .click();
  await expect(
    page.getByRole("button", { name: /Copied branch name/ }).first(),
  ).toBeVisible();

  await page.getByRole("button", { name: "Actions" }).first().click();
  await expect(
    page.getByRole("link", { name: "Activity" }).first(),
  ).toHaveAttribute("href", /\/branches\//);
  await expect(
    page.getByRole("link", { name: "Open tree" }).first(),
  ).toHaveAttribute("href", /\/tree\//);
  await expect(
    page.getByRole("button", { name: "Delete branch" }).first(),
  ).toBeDisabled();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("button", { name: "Delete branch" })).toHaveCount(
    0,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/branches-001-final-directory.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.treeRepositoryHref}/branches?tab=stale`);
  const viewportWidth = await page.locator("body").evaluate((body) => ({
    clientWidth: body.clientWidth,
    scrollWidth: body.scrollWidth,
  }));
  expect(viewportWidth.scrollWidth).toBeLessThanOrEqual(
    viewportWidth.clientWidth + 2,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/branches-001-final-mobile.jpg",
  });

  await context.clearCookies();
});
