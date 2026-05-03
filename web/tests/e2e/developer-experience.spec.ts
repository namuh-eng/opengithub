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

test("developer docs snippets are copyable and internal-only", async ({
  page,
}) => {
  await page.goto("/docs/api");
  await page.getByText("Request and response examples").first().click();
  await page.getByRole("button", { name: "Copy request" }).first().click();
  await expect(page.getByRole("status")).toContainText(/Copied|unavailable/);
  await expect(page.getByRole("link", { name: "Tokens" })).toHaveAttribute(
    "href",
    "/settings/tokens",
  );
  await expect(page.locator("article")).not.toContainText("api.github.com");

  await page.goto("/docs/git");
  await page.getByRole("button", { name: "Copy clone" }).click();
  await expect(page.getByRole("status")).toContainText(/Copied|unavailable/);
  await expect(
    page.getByRole("link", { name: "Token settings" }),
  ).toHaveAttribute("href", "/settings/tokens");
  await expect(page.locator("article")).not.toContainText("api.github.com");
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/dx-001-docs-copy.jpg",
  });
});
