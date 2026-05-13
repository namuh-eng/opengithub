import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededWiki = {
  cookieName: string;
  cookieValue: string;
  repositoryWikiHref: string;
};

function seedWiki(): SeededWiki {
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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededWiki;
}

async function signIn(page: Page, seeded: SeededWiki) {
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
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
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
      // make dev starts Next and Rust together; Next can win the readiness race.
    }
    await page.waitForTimeout(500);
  }
  throw new Error("Rust API did not become healthy for repository wiki E2E");
}

test.skip(
  !databaseUrl,
  "Repository Wiki E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in repository wiki supports Home, slug navigation, TOC expansion, clone copy, and mobile layout", async ({
  page,
}) => {
  test.setTimeout(60_000);
  const seeded = seedWiki();
  await signIn(page, seeded);

  await page.goto(seeded.repositoryWikiHref);
  await expect(
    page.getByRole("link", { exact: true, name: "Wiki" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("heading", { exact: true, name: "Home" }),
  ).toBeVisible();
  const wikiPages = page.getByRole("navigation", { name: "Wiki pages" });
  await expect(wikiPages).toBeVisible();
  await expect(page.getByText(/Clone this wiki locally/)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-001-final-home.jpg",
  });

  await wikiPages
    .getByRole("link", { exact: true, name: "Architecture Guide" })
    .click();
  await expect(page).toHaveURL(/\/wiki\/Architecture%20Guide$/);
  await expect(
    page.getByRole("heading", { exact: true, name: "Architecture Guide" }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("navigation", { name: "Wiki pages" })
      .getByRole("link", { exact: true, name: "Architecture Guide" }),
  ).toHaveAttribute("aria-current", "page");
  await page
    .getByRole("button", {
      name: "Expand Architecture Guide table of contents",
    })
    .click();
  await expect(
    page.getByRole("link", { name: "Services" }).first(),
  ).toBeVisible();
  await page.getByRole("link", { name: "Services" }).first().click();
  await expect(page).toHaveURL(/#services$/);
  await page.getByRole("button", { name: "Copy" }).click();
  await expect(page.getByRole("status")).toContainText("Copied URL");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-001-final-page.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(
    page.getByRole("heading", { exact: true, name: "Architecture Guide" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-001-final-mobile.jpg",
  });
});
