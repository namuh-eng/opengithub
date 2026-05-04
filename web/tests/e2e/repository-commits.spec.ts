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

async function waitForApiHealth(page: Page) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      const response = await page.request.get("http://localhost:3016/health", {
        timeout: 1000,
      });
      if (response.ok()) {
        return;
      }
    } catch {
      await page.waitForTimeout(500);
    }
  }
  throw new Error("Rust API did not become healthy for repository commits E2E");
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

async function createRepository(page: Page) {
  const repositoryName = `commit history ${Date.now().toString(36)}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page.getByLabel(/Description/).fill("Commit history Playwright smoke");
  await page
    .getByRole("combobox", { name: /Start with a template/ })
    .selectOption("rust-axum");
  await page.getByRole("button", { name: "Off" }).click();
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));
  return new URL(page.url()).pathname;
}

test.skip(
  !databaseUrl,
  "repository commits E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in commit history renders grouped rows and live links", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryHref = await createRepository(page);

  await page.goto(`${repositoryHref}/commits/main`);
  await expect(
    page.getByRole("heading", { name: "Commit history" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Initial commit/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/commit/`));
  await expect(page.getByRole("link", { name: /checks/ })).toHaveAttribute(
    "href",
    new RegExp(`${repositoryHref}/actions\\?commit=`),
  );
  await expect(
    page.getByRole("link", { name: /Browse repository at/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/tree/`));
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-phase2-default-history.jpg",
  });
});
