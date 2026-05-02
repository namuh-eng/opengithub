import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  actionsJobLogHref: string;
};

function seedSession(): SeededSession {
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
        ACTIONS_RUN_DETAIL_E2E: "1",
        DASHBOARD_E2E_EMPTY: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededSession;
}

async function signIn(page: Page, seeded: SeededSession) {
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
    () =>
      document.documentElement.scrollWidth >
      document.documentElement.clientWidth,
  );
  expect(overflow).toBe(false);
}

test.skip(
  !databaseUrl,
  "repository Actions job log E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in job log viewer renders job sidebar, steps, and annotations", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto(seeded.actionsJobLogHref);
  await expect(page.getByRole("heading", { name: "unit / web" })).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Workflow run jobs" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /deploy preview/ }),
  ).toHaveAttribute("href", /\/actions\/runs\/.*\/jobs\//);
  await expect(page.getByRole("textbox", { name: "Search log" })).toBeVisible();
  await expect(page.getByText("Installing dependencies")).toBeVisible();
  await expect(
    page.getByText("error: Expected string, found number"),
  ).toBeVisible();
  await expect(page.getByText("Type error")).toBeVisible();

  await page.getByRole("button", { name: /Job log/ }).click();
  await expect(
    page.getByText("error: Expected string, found number"),
  ).toBeHidden();
  await page.getByRole("button", { name: "Hide annotations" }).click();
  await expect(page.getByText("Problems in this job")).toBeHidden();
  await page.getByRole("button", { name: "Log options" }).click();
  await expect(page.getByRole("menu")).toContainText("rendered logs");
  await expect(
    page.getByRole("link", { name: "Download log" }),
  ).toHaveAttribute("href", /\/actions\/jobs\/.*\/logs\/download/);

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-004-phase2-job-viewer.jpg",
  });
});
