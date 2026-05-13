import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededNavigation = {
  cookieName: string;
  cookieValue: string;
  projectsWorkspaceHref: string;
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
        PROJECTS_WORKSPACE_E2E: "1",
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

async function openProjectFieldSettings(page: Page, seeded: SeededNavigation) {
  await page.goto(
    seeded.projectsWorkspaceHref.replace(/\/views\/\d+.*/, "/settings/fields"),
  );
  await expect(page.getByText("Project fields")).toBeVisible();
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

async function selectField(page: Page, name: RegExp) {
  const fieldLink = page
    .locator('a[href*="field="]')
    .filter({ hasText: name })
    .first();
  await expect(fieldLink).toBeVisible();
  await fieldLink.click();
}

test.skip(
  !databaseUrl,
  "Projects field settings E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects field settings support final signed-in administration smoke", async ({
  page,
}) => {
  test.setTimeout(90_000);
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openProjectFieldSettings(page, seeded);

  await expect(
    page.getByRole("link", { name: /Back to project/i }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "New field" })).toBeVisible();
  await expect(page.getByText(/used · .* remaining/i)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-field-list.jpg",
  });

  const uniqueField = `QA text ${Date.now()}`;
  await page.getByRole("button", { name: "New field" }).click();
  const newFieldDialog = page.getByRole("dialog", { name: "New field" });
  await expect(newFieldDialog).toBeVisible();
  await newFieldDialog.getByLabel("Name").fill(uniqueField);
  await newFieldDialog.getByLabel("Type").selectOption("text");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-create-dialog.jpg",
  });
  await newFieldDialog.getByRole("button", { name: "Create field" }).click();
  await expect(page.getByText("Field created.")).toBeVisible();
  await selectField(page, new RegExp(uniqueField));
  await page.getByLabel("Name", { exact: true }).fill(`${uniqueField} renamed`);
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByText("Field renamed.")).toBeVisible();
  await page.getByRole("button", { name: "Delete" }).click();
  await expect(
    page.getByRole("dialog", { name: /Delete .*renamed/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-delete-confirmation.jpg",
  });
  await page.getByRole("button", { name: "Delete field" }).click();
  await expect(page.getByText(/Field deleted/)).toBeVisible();

  await selectField(page, /Status|Priority|Stage/i);
  await page.getByPlaceholder("Ready").fill(`QA option ${Date.now()}`);
  await page.getByLabel("Color").selectOption("green");
  await page.getByPlaceholder("Optional").fill("Created by final smoke");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-single-select-options.jpg",
  });
  await page.getByRole("button", { name: "Add option" }).click();
  await expect(page.getByText("Option added.")).toBeVisible();
  const firstSaveOption = page
    .getByRole("button", { name: "Save option" })
    .first();
  await expect(firstSaveOption).toBeVisible();
  await firstSaveOption.click();
  await expect(page.getByText("Option saved.")).toBeVisible();
  const downButton = page
    .getByRole("button", { name: /Move .* option down/i })
    .first();
  if (await downButton.isEnabled()) {
    await downButton.click();
    await expect(page.getByText("Options reordered.")).toBeVisible();
  }

  await selectField(page, /Sprint|Cycle|Iteration/i);
  await expect(
    page.getByText(/Relative filters support @current/),
  ).toBeVisible();
  await page.getByLabel("Starts on", { exact: true }).fill("2026-05-04");
  await page.getByLabel("Duration", { exact: true }).fill("2");
  await page.getByRole("combobox", { name: /Unit/i }).selectOption("weeks");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-iteration-schedule.jpg",
  });
  await page.getByRole("button", { name: "Save schedule" }).click();
  await expect(page.getByText("Iteration schedule saved.")).toBeVisible();
  await page.getByRole("button", { name: "Add iteration" }).click();
  await expect(page.getByText("Iteration added.")).toBeVisible();
  await page.getByLabel("Break start date").fill("2026-06-15");
  await page.getByRole("button", { name: "Insert break" }).click();
  await expect(page.getByText("Break inserted.")).toBeVisible();

  await page.getByRole("link", { name: /Back to project/i }).click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  const filterBox = page.getByRole("searchbox", { name: /Filter items/i });
  await expect(filterBox).toBeVisible();
  await filterBox.fill("iteration:@current");
  await expect(page.getByText(/matching items/i)).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-filter-reflection.jpg",
  });

  await page.goto("/orgs/namuh/projects/1/settings/fields");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByText("Project fields")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-004-final-mobile.jpg",
  });
});
