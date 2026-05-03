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
  "repository Pages smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can mutate Pages settings and forbidden users do not see private metadata", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto(`${seeded.firstRepositoryHref}/settings/pages`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /Pages$/ }).first(),
  ).toBeVisible();
  await expect(page.getByText("Publishing source")).toBeVisible();
  await page.getByLabel("Source", { exact: true }).selectOption("branch");
  const branchOption = await page
    .getByLabel("Branch", { exact: true })
    .locator("option")
    .evaluateAll((options) =>
      options
        .map((option) => option.getAttribute("value") ?? "")
        .find((value) => value.length > 0),
    );
  expect(branchOption).toBeTruthy();
  await page
    .getByLabel("Branch", { exact: true })
    .selectOption(branchOption ?? "");
  await page.getByLabel("Folder", { exact: true }).selectOption("/");
  await page.getByRole("button", { name: "Save source" }).click();
  await expect(
    page.getByText("Branch source saved and a Pages deployment was queued."),
  ).toBeVisible();
  await expect(
    page.getByText(`${branchOption} · /(root)`).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Deploy saved source" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Deploy saved source" }).click();
  await expect(
    page.getByText("Pages deployment queued from the saved branch source."),
  ).toBeVisible();
  await expect(page.getByText("Domain and HTTPS")).toBeVisible();
  await page
    .getByLabel("Domain", { exact: true })
    .fill(`docs-${Date.now()}.example.com`);
  await page.getByRole("button", { name: "Save domain" }).click();
  await expect(page.getByText("Custom domain saved.")).toBeVisible();
  await expect(page.getByText("og-pages-", { exact: false })).toBeVisible();
  await page.getByRole("button", { name: "Recheck DNS" }).click();
  await expect(
    page.getByText("DNS verification rechecked from the Pages API."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enforce HTTPS" }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Unpublish Pages" }).click();
  await page.getByRole("button", { name: "Confirm unpublish" }).click();
  await expect(
    page.getByText("Pages unpublished. Repository files were preserved."),
  ).toBeVisible();
  await expect(page.getByText("Recent activity")).toBeVisible();
  await expect(
    page
      .locator("section")
      .filter({ hasText: "Recent activity" })
      .getByRole("link", { name: "Actions" }),
  ).toHaveAttribute("href", `${seeded.firstRepositoryHref}/actions`);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-phase3-pages-mutations.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
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
    path: "../ralph/screenshots/build/settings-006-phase3-pages-mobile.jpg",
  });

  await page.context().clearCookies();
  await signIn(page, seeded, seeded.profileActionCookieValue);
  await page.goto(`${seeded.firstRepositoryHref}/settings/pages`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Pages" }),
  ).toBeVisible();
  await expect(page.getByText("og-pages-")).toHaveCount(0);
  await expect(page.getByText("cloudfront", { exact: false })).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-006-phase3-pages-reader.jpg",
  });
});
