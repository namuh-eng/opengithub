import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededNavigation = {
  cookieName: string;
  cookieValue: string;
  projectsWorkspaceHref: string;
};

function seedNavigation(): SeededNavigation {
  if (!databaseUrl)
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
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

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

async function expectNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
}

test.skip(
  !databaseUrl,
  "Projects settings E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects settings support signed-in general status template access and lifecycle smoke", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const seeded = seedNavigation();
  await signIn(page, seeded);

  const settingsHref = seeded.projectsWorkspaceHref.replace(
    /\/views\/\d+.*/,
    "/settings",
  );
  await page.goto(settingsHref);
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.getByText("Project metadata")).toBeVisible();
  await expect(page.getByRole("link", { name: "Templates" })).toHaveAttribute(
    "href",
    /\/settings\/templates$/,
  );
  await expect(page.getByRole("link", { name: "Access" })).toHaveAttribute(
    "href",
    /\/settings\/access$/,
  );
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  const uniqueTitle = `QA settings ${Date.now()}`;
  await page.getByLabel("Title", { exact: true }).fill(uniqueTitle);
  await page
    .getByLabel("Short description")
    .fill("Updated by projects-007 E2E");
  await page
    .getByLabel("README Markdown")
    .fill("## QA settings\nVerified by browser smoke.");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByText("Project settings saved.")).toBeVisible();
  await expect(page.getByRole("heading", { name: uniqueTitle })).toBeVisible();

  await page.getByLabel("State").selectOption("complete");
  await page.getByLabel("Start date").fill("2026-05-01");
  await page.getByLabel("Target date").fill("2026-05-10");
  await page
    .getByLabel("Message")
    .fill("Browser smoke completed project settings.");
  await page.getByRole("button", { name: "Publish update" }).click();
  await expect(
    page.getByText("Project status update published."),
  ).toBeVisible();
  await expect(
    page
      .locator("p")
      .filter({ hasText: "Browser smoke completed project settings." }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Templates" }).click();
  await expect(page).toHaveURL(/\/settings\/templates$/);
  await expect(page.getByText("Copy-source settings")).toBeVisible();
  const templateTitle = `Template ${Date.now()}`;
  const templateCheckbox = page.getByLabel("Set this project as a template");
  if (!(await templateCheckbox.isChecked())) await templateCheckbox.check();
  await page.getByLabel("Template title").fill(templateTitle);
  await page
    .getByLabel("Copy-source information")
    .fill("Copy source verified from projects-007 E2E.");
  await page.getByLabel("Allow copies from visible users").check();
  await page.getByRole("button", { name: "Save template" }).click();
  await expect(page.getByText("Template settings saved.")).toBeVisible();
  await expect(page.getByText(/Template [0-9a-f-]{36}/)).toBeVisible();

  await page.getByRole("link", { name: "Access" }).click();
  await expect(page).toHaveURL(/\/settings\/access$/);
  await expect(page.getByText("Add collaborators or teams")).toBeVisible();
  await expect(page.getByRole("button", { name: "Add access" })).toBeVisible();
  await expectNoDeadControls(page);

  await page.getByRole("link", { name: "Danger Zone" }).click();
  await expect(page).toHaveURL(/\/settings\/danger$/);
  await expect(page.getByText("Project lifecycle")).toBeVisible();
  await page.getByRole("button", { name: "Close project" }).click();
  await expect(page.getByText("Project closed.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Reopen project" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Reopen project" }).click();
  await expect(page.getByText("Project reopened.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Close project" }),
  ).toBeVisible();
});
