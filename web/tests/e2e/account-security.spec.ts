import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

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
        ACCOUNT_SECURITY_E2E: "1",
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
  "account security E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("account security manages Google sign-in methods with sudo and last-identity protection", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto("/settings/security");
  await expect(
    page.getByRole("article").getByRole("heading", { name: "Security" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Sign-in methods" }),
  ).toBeVisible();
  await expect(page.getByText("Last identity")).toHaveCount(0);
  await expect(page.getByText("+second@opengithub.local")).toBeVisible();
  await expect(page.getByText("Sudo required")).toBeVisible();

  const emailInput = page.getByLabel("Account email");
  const email = await emailInput.getAttribute("placeholder");
  expect(email).toContain("@opengithub.local");
  await emailInput.fill(email ?? "");
  await page.getByRole("button", { name: "Enable sudo" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Sudo mode is active for this session.",
  );
  await expect(
    page.getByRole("link", { name: "Link Google account" }),
  ).toHaveAttribute("href", /\/api\/settings\/security\/google\/link/);

  const secondRow = page.locator(".list-row", {
    hasText: "+second@opengithub.local",
  });
  const secondEmail = await secondRow.locator(".t-sm").last().textContent();
  await secondRow.getByRole("button", { name: "Unlink" }).click();
  await expect(
    page.getByRole("button", { name: "Unlink sign-in method" }),
  ).toBeDisabled();
  await page
    .getByLabel(`Confirm unlink ${secondEmail ?? ""}`)
    .fill(secondEmail ?? "");
  await page.getByRole("button", { name: "Unlink sign-in method" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Google account unlinked.",
  );
  await expect(page.getByText("+second@opengithub.local")).toHaveCount(0);
  await expect(page.getByText("Last identity")).toBeVisible();
  await expect(page.getByRole("button", { name: "Unlink" })).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Configure 2FA" }),
  ).toBeDisabled();

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).not.toHaveCSS("overflow-x", "scroll");
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/security-001-account-security.jpg",
  });
});

test("active sessions can revoke devices and sign out everywhere except current", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto("/settings/security/sessions");
  await expect(
    page.getByRole("heading", { name: "Active sessions" }),
  ).toBeVisible();
  await expect(page.getByText("3 active sessions")).toBeVisible();
  await expect(page.getByText("Current", { exact: true })).toBeVisible();
  await expect(page.getByText("Windows PC · Chrome")).toBeVisible();
  await expect(page.getByText("iPhone · Safari")).toBeVisible();
  await expect(page.getByText("2001:db8::42")).toBeVisible();
  await expect(page.getByText("10.44.55.66")).toBeVisible();
  await expect(page.getByText("expired", { exact: false })).toHaveCount(0);

  const currentRow = page.locator("tr", { hasText: "Current" });
  await expect(
    currentRow.getByRole("button", { name: "Revoke" }),
  ).toBeDisabled();

  const mobileRow = page.locator("tr", { hasText: "iPhone · Safari" });
  await mobileRow.getByRole("button", { name: "Revoke" }).click();
  await expect(page.getByRole("status")).toContainText("Session revoked.");
  await expect(page.getByText("iPhone · Safari")).toHaveCount(0);
  await expect(page.getByText("2 active sessions")).toBeVisible();

  await page.keyboard.press("Tab");
  await expect(
    page.getByRole("button", { name: "Sign out everywhere" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Sign out everywhere" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Other sessions have been signed out.",
  );
  await expect(page.getByText("1 active session")).toBeVisible();
  await expect(page.getByText("Windows PC · Chrome")).toHaveCount(0);
  await expect(page.getByText("Current", { exact: true })).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).not.toHaveCSS("overflow-x", "scroll");
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/qa/security-002-active-sessions.png",
  });
});
