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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard) {
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

test.skip(
  !databaseUrl,
  "repository settings general smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can load and mutate repository general settings", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings`);

  await expect(page.getByRole("heading", { name: "General" })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /\/alpha-[a-f0-9]+/ }),
  ).toBeVisible();
  await expect(page.getByLabel("Repository name")).toHaveValue(/alpha-/);
  await expect(page.getByText("Repository state")).toBeVisible();
  await expect(page.getByText("Feature toggles")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Merge methods" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Destructive actions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "View branches" }),
  ).toHaveAttribute(
    "href",
    new RegExp(`${seeded.firstRepositoryHref}/branches$`),
  );

  const description = `Phase 3 settings mutation ${Date.now()}`;
  await page.getByLabel("Repository description").fill(description);
  await page.getByRole("button", { name: "Save profile" }).click();
  await expect(page.getByText("Repository profile saved.")).toBeVisible();
  await page.reload();
  await expect(page.getByLabel("Repository description")).toHaveValue(
    description,
  );

  const issues = page.getByLabel("Issues");
  const nextIssues = !(await issues.isChecked());
  await issues.setChecked(nextIssues);
  await page.getByRole("button", { name: "Save features" }).click();
  await expect(page.getByText("Feature toggles saved.")).toBeVisible();
  await page.reload();
  if (nextIssues) {
    await expect(page.getByLabel("Issues")).toBeChecked();
  } else {
    await expect(page.getByLabel("Issues")).not.toBeChecked();
  }

  await page.getByLabel("Allow squash merging").setChecked(false);
  await page.getByLabel("Allow merge commits").setChecked(false);
  await page.getByLabel("Allow rebase merging").setChecked(false);
  await expect(
    page.getByText("At least one merge method must remain enabled."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Save merge methods" }),
  ).toBeDisabled();
  await page.getByLabel("Allow squash merging").setChecked(true);
  await page.getByLabel("Default merge method").selectOption("squash");
  await page.getByRole("button", { name: "Save merge methods" }).click();
  await expect(page.getByText("Merge methods saved.")).toBeVisible();
  await page.reload();
  await expect(page.getByLabel("Allow squash merging")).toBeChecked();
  await expect(page.getByLabel("Default merge method")).toHaveValue("squash");

  await expect(
    page.getByRole("button", { name: "Archive repository" }),
  ).toBeDisabled();
  const repositoryFullName = seeded.firstRepositoryHref.slice(1);
  await page.getByLabel("Archive confirmation").fill(repositoryFullName);
  await page.getByRole("button", { name: "Archive repository" }).click();
  await expect(page.getByText("Repository archived.")).toBeVisible();
  await page.reload();
  await expect(page.locator(".chip", { hasText: "Archived" })).toBeVisible();
  await page
    .getByLabel("Repository description")
    .fill("Blocked while archived");
  await page.getByRole("button", { name: "Save profile" }).click();
  await expect(
    page.getByText(
      /archived repositories only allow unarchive settings updates/,
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Unarchive repository" }),
  ).toBeDisabled();
  await page.getByLabel("Archive confirmation").fill(repositoryFullName);
  await page.getByRole("button", { name: "Unarchive repository" }).click();
  await expect(page.getByText("Repository unarchived.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Delete repository unavailable" }),
  ).toBeDisabled();
  await page.setViewportSize({ width: 390, height: 900 });
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-001-phase4-general-guardrails.jpg",
  });

  await page.context().clearCookies();
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: seeded.profileActionCookieValue,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      sameSite: "Lax",
      secure: false,
    },
  ]);
  await page.goto(`${seeded.firstRepositoryHref}/settings`);
  await expect(
    page.getByRole("heading", { name: "Repository settings are restricted" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-001-phase4-forbidden.jpg",
  });
});
