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

test.skip(
  !databaseUrl,
  "repository Actions secrets smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can create update and delete Actions secrets and variables", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const suffix = Date.now().toString(36).toUpperCase();
  const secretName = `DEPLOY_KEY_${suffix}`;
  const variableName = `PUBLIC_BASE_URL_${suffix}`;

  await page.goto(`${seeded.firstRepositoryHref}/settings/secrets/actions`);
  await expect(
    page.getByRole("heading", { name: "Actions secrets" }),
  ).toBeVisible();

  await page.getByLabel("Name").fill(secretName);
  await page.getByLabel("Secret value").fill(`super-secret-${suffix}`);
  await page.getByRole("button", { name: "Add secret" }).click();
  await expect(page.getByText(`${secretName} created.`)).toBeVisible();
  await expect(page.getByText(secretName, { exact: true })).toBeVisible();
  await expect(page.getByText("Write-only values")).toBeVisible();
  await expect(page.getByText(`super-secret-${suffix}`)).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-populated.jpg",
  });

  await page.reload();
  await expect(page.getByText(secretName, { exact: true })).toBeVisible();
  await expect(page.getByText(`super-secret-${suffix}`)).toHaveCount(0);
  const secretRow = page
    .getByText(secretName, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'list-row')][1]");
  await secretRow.getByRole("button", { name: "Update" }).click();
  await secretRow.getByLabel("Secret value").fill(`rotated-secret-${suffix}`);
  await secretRow.getByRole("button", { name: "Update secret" }).click();
  await expect(page.getByText(`${secretName} updated.`)).toBeVisible();
  await expect(page.getByText(`rotated-secret-${suffix}`)).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-phase3-secrets-mutations.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-mutation.jpg",
  });

  await page.getByRole("link", { name: /Variables/ }).click();
  await expect(page).toHaveURL(/tab=variables/);
  await page.getByLabel("Name").fill(variableName);
  await page.getByLabel("Variable value").fill("https://opengithub.namuh.co");
  await page.getByRole("button", { name: "Add variable" }).click();
  await expect(page.getByText(`${variableName} created.`)).toBeVisible();
  await expect(page.getByText(variableName, { exact: true })).toBeVisible();
  await expect(page.getByText("https://opengithub.namuh.co")).toBeVisible();

  const variableRow = page
    .getByText(variableName, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'list-row')][1]");
  await variableRow.getByRole("button", { name: "Update" }).click();
  await variableRow
    .getByLabel("Variable value")
    .fill("https://staging.opengithub.namuh.co");
  await variableRow.getByRole("button", { name: "Update variable" }).click();
  await expect(page.getByText(`${variableName} updated.`)).toBeVisible();
  await expect(
    page.getByText("https://staging.opengithub.namuh.co"),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-variable.jpg",
  });

  await variableRow.getByRole("button", { name: "Delete" }).click();
  await variableRow
    .getByLabel(`Confirm delete ${variableName}`)
    .fill(variableName);
  await variableRow.getByRole("button", { name: "Delete variable" }).click();
  await expect(page.getByText(`${variableName} deleted.`)).toBeVisible();
  await expect(
    page.locator(".list-row").filter({ hasText: variableName }),
  ).toHaveCount(0);

  await page
    .getByRole("navigation", { name: "Secrets and variables tabs" })
    .getByRole("link", { name: /Secrets/ })
    .click();
  const refreshedSecretRow = page
    .getByText(secretName, { exact: true })
    .locator("xpath=ancestor::div[contains(@class, 'list-row')][1]");
  await refreshedSecretRow.getByRole("button", { name: "Delete" }).click();
  await refreshedSecretRow
    .getByLabel(`Confirm delete ${secretName}`)
    .fill(secretName);
  await refreshedSecretRow
    .getByRole("button", { name: "Delete secret" })
    .click();
  await expect(page.getByText(`${secretName} deleted.`)).toBeVisible();
  await expect(
    page.locator(".list-row").filter({ hasText: secretName }),
  ).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-empty.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Actions secrets" }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    )
    .toBe(true);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-mobile.jpg",
  });

  await page.context().clearCookies();
  await signIn(page, seeded, seeded.profileActionCookieValue);
  await page.goto(`${seeded.firstRepositoryHref}/settings/secrets/actions`);
  await expect(
    page.getByRole("heading", { name: "Actions secrets are restricted" }),
  ).toBeVisible();
  await expect(page.getByText(`DEPLOY_KEY_${suffix}`)).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-005-final-secrets-forbidden.jpg",
  });
});
