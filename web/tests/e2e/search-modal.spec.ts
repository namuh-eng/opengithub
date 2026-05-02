import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
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
        SEARCH_E2E_MARKER: "searchmodal",
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
  "search modal E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("global search modal supports shortcut, suggestions, saved searches, and Escape", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/dashboard");
  await page.keyboard.press("/");
  const searchDialog = page.getByRole("dialog", { name: "Search" });
  const combobox = searchDialog.getByRole("combobox", {
    name: "Search opengithub",
  });
  await expect(searchDialog).toBeVisible();
  await expect(combobox).toBeFocused();
  await expectNoDeadControls(page);

  await combobox.fill("language:ru");
  await searchDialog.getByRole("option", { name: /language:rust/ }).click();
  await expect(combobox).toHaveValue("language:rust ");

  await searchDialog.getByRole("button", { name: "path:src/" }).click();
  await expect(combobox).toHaveValue("language:rust path:src/ ");

  await combobox.fill("searchmodal");
  await expect(
    searchDialog.getByRole("option", { name: /searchmodal/ }).first(),
  ).toBeVisible();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/search\?q=searchmodal&type=repositories$/);

  await page.goto("/dashboard");
  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  await expect(searchDialog).toBeVisible();
  await combobox.fill("searchmodal language:rust");
  await searchDialog
    .getByRole("button", { name: "Create saved search" })
    .click();
  const createDialog = page.getByRole("dialog", {
    name: "Create saved search",
  });
  await expect(createDialog).toBeVisible();
  await expect(createDialog.getByLabel("Name")).toBeFocused();
  await createDialog
    .getByRole("button", { name: "Create saved search" })
    .click();
  await expect(createDialog.getByRole("alert")).toContainText(
    "Name is required.",
  );

  const savedName = `Guardrail search ${Date.now()}`;
  await createDialog.getByLabel("Name").fill(savedName);
  await createDialog.getByLabel("Query").fill("searchmodal language:rust");
  await createDialog
    .getByRole("button", { name: "Create saved search" })
    .click();
  await expect(searchDialog.getByRole("status")).toContainText(
    `Saved "${savedName}".`,
  );
  await expect(
    searchDialog.getByRole("option", { name: savedName }),
  ).toBeVisible();

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-002-final-desktop-modal.jpg",
  });

  await page.keyboard.press("Escape");
  await expect(searchDialog).toHaveCount(0);
});

test("global search modal keeps mobile viewport within bounds", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto("/dashboard");
  await page.keyboard.press("/");
  const searchDialog = page.getByRole("dialog", { name: "Search" });
  await expect(searchDialog).toBeVisible();
  await searchDialog
    .getByRole("combobox", { name: "Search opengithub" })
    .fill("searchmodal");
  await expect(
    searchDialog.getByRole("option", { name: /searchmodal/ }).first(),
  ).toBeVisible();

  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-002-final-mobile-modal.jpg",
  });
});
