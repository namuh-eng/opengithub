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
  "developer experience E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("developer settings expose copyable opengithub token workflows", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto("/settings/tokens");

  await expect(
    page.getByRole("heading", {
      exact: true,
      name: "Personal access tokens",
    }),
  ).toBeVisible();
  await expect(page.getByText("Your personal access tokens")).toBeVisible();
  await expect(page.getByText("No personal access tokens yet")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New fine-grained token" }),
  ).toHaveAttribute(
    "href",
    "/settings/personal-access-tokens/new?type=fine_grained",
  );
  await expect(
    page.getByRole("link", { name: "New classic token" }),
  ).toHaveAttribute(
    "href",
    "/settings/personal-access-tokens/new?type=classic",
  );
  await page.getByText("Generate new token").click();
  await expect(
    page.getByRole("link", { name: /Fine-grained token/ }),
  ).toHaveAttribute(
    "href",
    "/settings/personal-access-tokens/new?type=fine_grained",
  );
  await expect(
    page.getByRole("link", { name: /Classic token/ }),
  ).toHaveAttribute(
    "href",
    "/settings/personal-access-tokens/new?type=classic",
  );
  await expect(page.getByText("Token quickstart")).toBeVisible();
  await expect(page.getByText("repo:read")).toBeVisible();
  await expect(page.getByText("api:write")).toBeVisible();
  await expect(page.locator("article")).not.toContainText("api.github.com");
  await expect(
    page.getByRole("link", { name: "REST API endpoint catalog" }),
  ).toHaveAttribute("href", "/docs/api");
  await expect(
    page.getByRole("link", { name: "Git over HTTPS guide" }),
  ).toHaveAttribute("href", "/docs/git");

  await page.getByRole("button", { name: "Copy API curl" }).click();
  await expect(page.getByRole("status")).toContainText(/Copied|unavailable/);
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-001-phase2-token-list.jpg",
  });
});

test("fine-grained token creation reveals the secret once and persists prefix metadata", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const tokenName = `E2E deploy ${Date.now().toString(36)}`;

  await page.goto(
    `/settings/personal-access-tokens/new?name=${encodeURIComponent(tokenName)}&contents=read`,
  );

  await expect(
    page.getByRole("heading", { level: 1, name: "New fine-grained token" }),
  ).toBeVisible();
  const emailInput = page.getByLabel("Account email");
  const email = await emailInput.getAttribute("placeholder");
  expect(email).toContain("@opengithub.local");
  await emailInput.fill(email ?? "");
  await page.getByRole("button", { name: "Enable sudo" }).click();
  await expect(
    page.getByText("Sudo mode is active for this session."),
  ).toBeVisible();

  await page.getByLabel("No repository access").check();
  await page.getByRole("button", { name: "Generate token" }).click();

  await expect(
    page.getByText("Token created. Copy it now; it will not be shown again."),
  ).toBeVisible();
  const reveal = page.locator("code").filter({ hasText: /^oghp_/ });
  await expect(reveal).toBeVisible();
  const plainTextToken = (await reveal.textContent()) ?? "";
  expect(plainTextToken.startsWith("oghp_")).toBeTruthy();
  await page.getByRole("button", { name: "Copy token" }).click();
  await expect(page.getByText(/Copied|Copy unavailable/)).toBeVisible();

  await page.getByRole("link", { name: "Return to token list" }).click();
  await expect(page.getByText(tokenName)).toBeVisible();
  await expect(page.locator("main")).toContainText(plainTextToken.slice(0, 17));
  await expect(page.locator("main")).not.toContainText(plainTextToken);

  const tokenRow = page.locator(".list-row", { hasText: tokenName });
  await tokenRow.getByRole("button", { name: "Revoke" }).click();
  await expect(page.getByText(`Revoke ${tokenName}`)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Revoke token" }),
  ).toBeDisabled();
  await page.getByLabel(`Confirm revoke ${tokenName}`).fill(tokenName);
  await page.getByRole("button", { name: "Revoke token" }).click();
  await expect(page.getByRole("status")).toContainText(`${tokenName} revoked.`);
  await expect(page.getByText("Revoked", { exact: true })).toBeVisible();
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-001-phase4-token-revoke.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/settings/tokens");
  await expect(
    page.getByRole("heading", {
      exact: true,
      name: "Personal access tokens",
    }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth + 2,
  );
  expect(horizontalOverflow).toBeFalsy();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-001-final-token-list-mobile.jpg",
  });
});

test("classic token creation uses broad legacy scopes", async ({ page }) => {
  const seeded = seedSession();
  await signIn(page, seeded);

  await page.goto(
    "/settings/personal-access-tokens/new?type=classic&name=E2E%20classic&contents=write&packages=write&api=read",
  );

  await expect(
    page.getByRole("heading", { level: 1, name: "New classic token" }),
  ).toBeVisible();
  const emailInput = page.getByLabel("Account email");
  const email = await emailInput.getAttribute("placeholder");
  await emailInput.fill(email ?? "");
  await page.getByRole("button", { name: "Enable sudo" }).click();
  await expect(
    page.getByText("Sudo mode is active for this session."),
  ).toBeVisible();
  await expect(page.getByText(/Classic tokens use broad access/)).toBeVisible();

  await page.getByRole("button", { name: "Generate token" }).click();
  await expect(
    page.getByText("Token created. Copy it now; it will not be shown again."),
  ).toBeVisible();
  await page.getByRole("link", { name: "Return to token list" }).click();
  await expect(page.getByText("E2E classic")).toBeVisible();
  await expect(page.getByText("Classic", { exact: true })).toBeVisible();
  await expect(page.getByText("All accessible repositories")).toBeVisible();
  await expectNoDeadControls(page);
});

test("developer docs snippets are copyable and internal-only", async ({
  page,
}) => {
  await page.goto("/docs/api");
  await expect(
    page.locator("code").filter({ hasText: /^\/api\/settings\/tokens\/new$/ }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({ hasText: /^\/api\/settings\/sudo$/ }),
  ).toBeVisible();
  await expect(
    page
      .locator("code")
      .filter({ hasText: /^\/api\/settings\/tokens\/\{token_id\}$/ }),
  ).toBeVisible();
  await expect(
    page.getByText(/returns the plaintext secret exactly once/i),
  ).toBeVisible();
  await page.getByText("Request and response examples").first().click();
  await page.getByRole("button", { name: "Copy request" }).first().click();
  await expect(page.getByRole("status")).toContainText(/Copied|unavailable/);
  await expect(page.getByRole("link", { name: "Tokens" })).toHaveAttribute(
    "href",
    "/settings/tokens",
  );
  await expect(page.locator("article")).not.toContainText("api.github.com");

  await page.goto("/docs/git");
  await expect(page.getByText("Authenticate with a token")).toBeVisible();
  await expect(page.getByText("REST and packages")).toBeVisible();
  await expect(page.getByText("repo:read").last()).toBeVisible();
  await expect(page.getByText("repo:write")).toBeVisible();
  await expect(
    page.getByText(/Revoked or expired tokens fail immediately/),
  ).toBeVisible();
  await page.getByRole("button", { name: "Copy clone" }).click();
  await expect(page.getByRole("status").last()).toContainText(
    /Copied|unavailable/,
  );
  await page.getByRole("button", { name: "Copy authenticated clone" }).click();
  await expect(page.getByRole("status").last()).toContainText(
    /Copied|unavailable/,
  );
  await page.getByRole("button", { name: "Copy automation auth" }).click();
  await expect(page.getByRole("status").last()).toContainText(
    /Copied|unavailable/,
  );
  await expect(
    page.getByRole("link", { name: "Token settings" }),
  ).toHaveAttribute("href", "/settings/tokens");
  await expect(page.locator("article")).not.toContainText("api.github.com");
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-001-final-docs-git.jpg",
  });

  await page.goto("/docs/api");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/credentials-001-final-docs-api.jpg",
  });
});
