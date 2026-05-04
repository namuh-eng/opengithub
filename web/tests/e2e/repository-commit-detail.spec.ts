import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  treeRepositoryHref?: string;
};

function seedSession(extraEnv: Record<string, string> = {}): SeededSession {
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
        ...extraEnv,
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
  throw new Error("Rust API did not become healthy for commit detail E2E");
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test.skip(
  !databaseUrl,
  "repository commit detail E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in commit detail renders summary controls and unified diff", async ({
  page,
}) => {
  const seeded = seedSession({ DASHBOARD_E2E_TREE_REFS: "1" });
  await signIn(page, seeded);
  expect(seeded.treeRepositoryHref).toBeTruthy();
  const repositoryHref = seeded.treeRepositoryHref as string;

  await page.goto(`${repositoryHref}/commits/main`);
  const commitLink = page.getByRole("link", { name: /Initial commit/ }).first();
  const detailHref = await commitLink.getAttribute("href");
  expect(detailHref).toMatch(new RegExp(`${repositoryHref}/commit/`));
  await commitLink.click();
  await expect(page).toHaveURL(new RegExp(`${repositoryHref}/commit/`));
  await expect(
    page.getByRole("heading", { name: /Initial commit/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "Browse files" }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/tree/`));
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", `${repositoryHref}/commits/main`);
  await expect(page.getByRole("link", { name: /checks/i })).toHaveAttribute(
    "href",
    new RegExp(`${repositoryHref}/actions\\?commit=`),
  );
  await page.getByRole("button", { name: "Copy full SHA" }).click();
  await expect(page.getByRole("status")).toContainText(/copied|unavailable/i);
  await expect(
    page.getByRole("heading", { name: "Changed files" }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Changed file tree" }),
  ).toBeVisible();
  await expect(page.getByText(/files changed with/)).toBeVisible();
  await expect(page.locator("article.card[id^='diff-']").first()).toBeVisible();
  await expect(page.getByRole("link", { name: "Raw" }).first()).toHaveAttribute(
    "href",
    new RegExp(`${repositoryHref}/raw/`),
  );
  await expect(
    page.getByRole("link", { name: "View file" }).first(),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/blob/`));
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-002-phase2-diff.jpg",
  });
});
