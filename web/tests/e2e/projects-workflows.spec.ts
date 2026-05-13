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

async function openFirstProjectWorkflowSettings(
  page: Page,
  seeded: SeededNavigation,
) {
  expect(seeded.projectsWorkspaceHref).toBeTruthy();
  await page.goto(
    seeded.projectsWorkspaceHref.replace(/\/views\/\d+.*/, "/workflows"),
  );
  await expect(page.getByText("Project workflows")).toBeVisible();
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

test.skip(
  !databaseUrl,
  "Projects workflows E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects workflows support final signed-in automation smoke", async ({
  page,
}) => {
  test.setTimeout(90_000);
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openFirstProjectWorkflowSettings(page, seeded);

  await expect(
    page.getByRole("link", { name: "Back to project" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Fields" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Workflows" })).toBeVisible();
  await expect(
    page.getByText("@opengithub-project-automation").first(),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-006-final-workflows-list.jpg",
  });

  await page.getByRole("button", { name: "Edit" }).first().click();
  await expect(page.getByText("Workflow editor")).toBeVisible();
  await expect(page.getByLabel("Event")).toBeVisible();
  await expect(page.getByLabel("Condition")).toBeVisible();
  await expect(page.getByLabel("Target field")).toBeVisible();
  await expect(page.getByLabel("Target value")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-006-final-edit-dialog.jpg",
  });

  const condition = page.getByLabel("Condition");
  if (await condition.isEnabled()) {
    await condition.fill("state:closed label:ready");
    const targetField = page.getByLabel("Target field");
    if (await targetField.isEnabled()) {
      const options = await targetField.locator("option").all();
      if (options.length > 1) {
        await targetField.selectOption({ index: 1 });
      }
    }
    const targetValue = page.getByLabel("Target value");
    if (await targetValue.isEnabled()) {
      const options = await targetValue.locator("option").all();
      if (options.length > 1) {
        await targetValue.selectOption({ index: 1 });
      }
    }
    const repositoryTarget = page
      .locator('label:has(input[type="checkbox"])')
      .first();
    if (await repositoryTarget.isVisible()) {
      await repositoryTarget.scrollIntoViewIfNeeded();
      await page.screenshot({
        fullPage: true,
        path: "../ralph/screenshots/build/projects-006-final-repository-selector.jpg",
      });
    }
    await page.getByLabel("Archive criteria").fill("14");
    await page.screenshot({
      fullPage: true,
      path: "../ralph/screenshots/build/projects-006-final-archive-criteria.jpg",
    });
    await page.getByRole("button", { name: "Save workflow" }).click();
    await expect(page.getByRole("status")).toHaveText(
      "Workflow configuration saved.",
    );
  } else {
    await expect(page.getByText("Read-only")).toBeVisible();
    await page.screenshot({
      fullPage: true,
      path: "../ralph/screenshots/build/projects-006-final-read-only.jpg",
    });
  }

  await page.getByRole("button", { name: "Close" }).click();
  await expect(
    page.getByRole("heading", { name: "Recent automation" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-006-final-activity-log.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByText("Project workflows")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-006-final-mobile.jpg",
  });
});
