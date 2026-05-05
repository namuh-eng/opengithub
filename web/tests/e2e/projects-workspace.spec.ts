import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededNavigation = {
  cookieName: string;
  cookieValue: string;
};

function seedNavigation(): SeededNavigation {
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
  return JSON.parse(output) as SeededNavigation;
}

async function signIn(page: Page, seeded: SeededNavigation) {
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

async function expectNoHorizontalOverflow(page: Page) {
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
}

async function openFirstProjectWorkspace(page: Page) {
  await page.goto("/orgs/namuh/projects");
  await expect(page.getByRole("heading", { name: /Projects/i })).toBeVisible();
  const workspaceLink = page
    .locator('a[href*="/projects/"][href*="/views/"]')
    .first();
  await expect(workspaceLink).toBeVisible();
  await workspaceLink.click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
}

test.skip(
  !databaseUrl,
  "Projects workspace E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects workspace table supports saved views, edits, add row, and final screenshots", async ({
  page,
}) => {
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openFirstProjectWorkspace(page);

  await expect(
    page.getByRole("navigation", { name: /Project views/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("searchbox", { name: /Filter items/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /View configuration/i }),
  ).toBeVisible();
  await expect(page.getByRole("table")).toBeVisible();
  await expect(page.getByText(/matching items/i)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-002-final-default-table.jpg",
  });

  await page.getByRole("button", { name: /View configuration/i }).click();
  await expect(
    page.getByRole("group", { name: /Visible fields/i }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /Save view/i })).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-002-final-view-config.jpg",
  });
  await page
    .getByRole("button", { name: /Cancel|Revert/i })
    .first()
    .click();

  const editableCell = page
    .getByRole("button", { name: /Edit .* field/i })
    .first();
  await expect(editableCell).toBeVisible();
  await editableCell.click();
  await expect(page.getByRole("button", { name: /Save field/i })).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-002-final-inline-editor.jpg",
  });
  await page.getByRole("button", { name: /Cancel field/i }).click();

  await page.getByRole("button", { name: /Add linked item/i }).click();
  await expect(
    page.getByRole("textbox", { name: /Issue or pull request URL/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-002-final-add-row.jpg",
  });
  await page.keyboard.press("Escape");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-002-final-mobile.jpg",
  });
});
