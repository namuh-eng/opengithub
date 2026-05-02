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
  "repository branch settings smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can create, edit, and delete branch policies", async ({ page }) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const suffix = Date.now().toString(36);

  await page.goto(`${seeded.firstRepositoryHref}/settings/branches`);
  await expect(page.getByRole("heading", { name: "Branches" })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Rules and rulesets" }),
  ).toBeVisible();

  await page
    .getByRole("button", { name: "New branch protection rule" })
    .first()
    .click();
  await page.getByLabel("Branch pattern").fill(`phase3-${suffix}`);
  await page.getByLabel("Description").fill("Phase 3 browser-created rule.");
  await page.getByLabel("Required reviews").fill("1");
  await page.getByLabel("Required status checks").fill("ci\nbiome");
  await page.getByLabel("Require signed commits").check();
  await page.getByRole("button", { name: "Create policy" }).click();
  await expect(page.getByText("Branch policy saved.")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: `phase3-${suffix}` }),
  ).toBeVisible();

  await page
    .locator("article", { hasText: `phase3-${suffix}` })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Required reviews").fill("2");
  await page.getByLabel("Require linear history").check();
  await page.getByRole("button", { name: "Save policy" }).click();
  await expect(page.getByText("2+ reviews")).toBeVisible();
  await expect(page.getByText("Linear history")).toBeVisible();

  await page.getByRole("button", { name: "New ruleset" }).click();
  await page.getByLabel("Ruleset name").fill(`Evaluate ${suffix}`);
  await page.getByLabel("Branch patterns").fill(`release-${suffix}/*`);
  await page.getByLabel("Evaluate").check();
  await page.getByLabel("Required status checks").fill("release-smoke");
  await page.getByRole("button", { name: "Create policy" }).click();
  await expect(page.getByText(`Evaluate ${suffix}`)).toBeVisible();
  await expect(
    page
      .locator("article", { hasText: `Evaluate ${suffix}` })
      .getByText("Evaluate", { exact: true }),
  ).toBeVisible();

  await page
    .locator("article", { hasText: `phase3-${suffix}` })
    .getByRole("button", { name: "Delete" })
    .click();
  await expect(page.getByRole("alertdialog")).toBeVisible();
  await page.getByRole("button", { name: "Delete policy" }).click();
  await expect(page.getByText("Branch policy saved.")).toBeVisible();
  await expect(page.getByText(`phase3-${suffix}`)).toHaveCount(0);

  await page.reload();
  await expect(page.getByText(`Evaluate ${suffix}`)).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-003-phase3-branches-mutations.jpg",
  });

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-003-final-branches-admin.jpg",
  });

  await page.setViewportSize({ width: 390, height: 860 });
  await page.goto(`${seeded.firstRepositoryHref}/settings/branches`);
  await expect(page.getByRole("heading", { name: "Branches" })).toBeVisible();
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-003-final-branches-mobile.jpg",
  });

  await page.context().clearCookies();
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: seeded.profileActionCookieValue,
    },
  ]);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(`${seeded.firstRepositoryHref}/settings/branches`);
  if (
    (await page
      .getByRole("heading", {
        name: "Repository branch policies are restricted",
      })
      .count()) > 0
  ) {
    await expect(
      page.getByRole("heading", {
        name: "Repository branch policies are restricted",
      }),
    ).toBeVisible();
  } else {
    await expect(page.getByRole("heading", { name: "Branches" })).toBeVisible();
    await expect(
      page.getByText(/Read-only|editing requires admin access/i).first(),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "New branch protection rule" }),
    ).toHaveCount(0);
  }
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-003-final-branches-readonly.jpg",
  });
});
