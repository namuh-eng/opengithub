import { execFileSync } from "node:child_process";
import { expect, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
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
        DASHBOARD_E2E_EMPTY: "0",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededSession;
}

test.skip(
  !databaseUrl,
  "Actions runners E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("repository admin manages Actions runners and scheduling controls", async ({
  page,
}) => {
  const seeded = seedSession();
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

  await page.goto(`${seeded.firstRepositoryHref}/settings/actions`);
  await expect(
    page.getByRole("heading", { name: "Actions", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Self-hosted runners")).toBeVisible();
  await page.getByLabel("Runner name").fill("linux-build-1");
  await page.getByLabel("Labels").fill("self-hosted, ubuntu-latest");
  await page.getByRole("button", { name: "Register runner" }).click();
  await expect(page.getByText("Runner registered.")).toBeVisible();
  await page.getByLabel("Concurrency limit").fill("8");
  await page
    .getByLabel("Cancel older in-progress runs in the same concurrency group")
    .check();
  await page.getByRole("button", { name: "Save scheduling settings" }).click();
  await expect(
    page.getByText("Actions workflow settings saved."),
  ).toBeVisible();
  await expect(page.getByText("Secret release approval")).toBeVisible();
  await page.getByRole("textbox", { name: "Environment" }).fill("production");
  await page
    .getByLabel(
      "Require reviewer approval before environment secrets are released",
    )
    .check();
  await page
    .getByRole("button", { name: "Save environment protection" })
    .click();
  await expect(
    page.getByText("Actions workflow settings saved."),
  ).toBeVisible();
  await expect(page.getByText("production: approval required")).toBeVisible();
  await page.getByRole("button", { name: "Assign queued jobs" }).click();
  await expect(page.getByText(/queued jobs? assigned/)).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth,
    ),
  ).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-006-runners-e2e.png",
  });
});
