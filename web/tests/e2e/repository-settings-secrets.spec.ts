import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;
const apiUrl = process.env.API_URL ?? "http://localhost:3016";

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

async function signIn(page: Page, seeded: SeededDashboard, value?: string) {
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: value ?? seeded.cookieValue,
    },
  ]);
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
    if (await button.isDisabled()) {
      await expect(button).toHaveAttribute("aria-disabled", "true");
    }
  }
}

function repoApiPath(seeded: SeededDashboard, suffix: string) {
  const [, owner, repo] = seeded.firstRepositoryHref.split("/");
  return `${apiUrl}/api/repos/${owner}/${repo}/settings/secrets/${suffix}`;
}

test.skip(
  !databaseUrl,
  "repository Actions secrets smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can view write-only Actions secrets and variables metadata", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const suffix = Date.now().toString(36).toUpperCase();

  const secretResponse = await page.request.post(
    repoApiPath(seeded, "secrets"),
    {
      data: { name: `DEPLOY_KEY_${suffix}`, value: `super-secret-${suffix}` },
      headers: { cookie },
    },
  );
  expect(secretResponse.ok(), await secretResponse.text()).toBe(true);

  const variableResponse = await page.request.post(
    repoApiPath(seeded, "variables"),
    {
      data: {
        name: `PUBLIC_BASE_URL_${suffix}`,
        value: "https://opengithub.namuh.co",
      },
      headers: { cookie },
    },
  );
  expect(variableResponse.ok(), await variableResponse.text()).toBe(true);

  await page.goto(`${seeded.firstRepositoryHref}/settings/secrets`);
  await expect(
    page.getByRole("heading", { name: "Secrets and variables" }),
  ).toBeVisible();
  await expect(page.getByText(`DEPLOY_KEY_${suffix}`)).toBeVisible();
  await expect(page.getByText("Write-only values")).toBeVisible();
  await expect(page.getByText(`super-secret-${suffix}`)).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-phase2-secrets-shell.jpg",
  });

  await page.getByRole("link", { name: /Variables/ }).click();
  await expect(page).toHaveURL(/tab=variables/);
  await expect(page.getByText(`PUBLIC_BASE_URL_${suffix}`)).toBeVisible();
  await expect(page.getByText("https://opengithub.namuh.co")).toBeVisible();
  await expectNoDeadControls(page);

  await page.context().clearCookies();
  await signIn(page, seeded, seeded.profileActionCookieValue);
  await page.goto(`${seeded.firstRepositoryHref}/settings/secrets`);
  await expect(
    page.getByRole("heading", { name: "Actions secrets are restricted" }),
  ).toBeVisible();
  await expect(page.getByText(`DEPLOY_KEY_${suffix}`)).toHaveCount(0);
  await expectNoDeadControls(page);
});
