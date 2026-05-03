import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

const validSshPublicKey =
  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIPhY2XwcvYPGAilZzICTAgSiG3kOTaMAP1+y/4U9HQb6 phase2@example";

type SeededSession = {
  cookieName: string;
  cookieValue: string;
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

test.skip(
  !databaseUrl,
  "developer keys E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("SSH key settings create and delete a public key", async ({ page }) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const keyTitle = `E2E SSH ${Date.now().toString(36)}`;

  await page.goto("/settings/keys");

  await expect(
    page.getByRole("heading", { exact: true, name: "SSH keys" }).first(),
  ).toBeVisible();
  await expect(page.getByText("Authentication keys")).toBeVisible();
  await page.getByRole("button", { name: "New SSH key" }).first().click();
  await page.getByLabel("Title").fill(keyTitle);
  await page.getByLabel("Public key").fill(validSshPublicKey);
  await page.getByRole("button", { name: "Add SSH key" }).click();
  await expect(page.getByRole("status")).toContainText(`${keyTitle} added.`);
  await expect(page.getByText("SHA256:")).toBeVisible();
  await expect(page.locator("main")).not.toContainText("PRIVATE KEY");

  const row = page.locator(".list-row", { hasText: keyTitle });
  await row.getByRole("button", { name: "Delete" }).click();
  const emailInput = page.getByLabel("Account email");
  const email = await emailInput.getAttribute("placeholder");
  expect(email).toContain("@opengithub.local");
  await emailInput.fill(email ?? "");
  await page.getByRole("button", { name: "Enable sudo" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Sudo mode is active for this session.",
  );
  await page.getByLabel(`Confirm delete ${keyTitle}`).fill(keyTitle);
  await page.getByRole("button", { name: "Delete SSH key" }).click();
  await expect(page.getByRole("status")).toContainText(`${keyTitle} deleted.`);
  await expect(page.getByText("Deleted", { exact: true })).toBeVisible();
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-002-phase2-ssh-keys.jpg",
  });
});
