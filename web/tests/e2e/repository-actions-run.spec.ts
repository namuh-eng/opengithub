import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  actionsRunDetailHref: string;
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
  "repository Actions run E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in workflow run detail renders jobs, annotations, and artifacts", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto(seeded.actionsRunDetailHref);
  await expect(
    page.getByRole("heading", { name: /Validate Editorial CI/ }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("navigation", { name: "Repository" })
      .getByRole("link", { name: "Actions" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Editorial CI" })).toBeVisible();
  await expect(page.getByText("Workflow Dispatch on")).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Workflow run jobs" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: /Attempt 2/ })).toBeVisible();
  await expect(page.getByText("Type error")).toBeVisible();
  await expect(page.getByText("Expected string, found number")).toBeVisible();
  await expect(page.getByText("playwright-report")).toBeVisible();
  await expect(page.getByText("sha256:abc123")).toBeVisible();

  await page.getByRole("link", { name: /deploy preview/ }).click();
  await expect(
    page.getByRole("heading", { exact: true, name: "deploy preview" }),
  ).toBeVisible();
  await expect(page.getByText("Logs deleted")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-003-phase2-run-detail.jpg",
  });
});
