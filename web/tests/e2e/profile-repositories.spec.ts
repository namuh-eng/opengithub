import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededProfile = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
};

function seedProfile(): SeededProfile {
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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededProfile;
}

async function signIn(page: Page, seeded: SeededProfile) {
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

function profileHref(seeded: SeededProfile) {
  const [, owner] = seeded.firstRepositoryHref.split("/");
  return `/${owner}`;
}

test.skip(
  !databaseUrl,
  "profile repository E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("profile repositories tab renders real rows and filter controls", async ({
  page,
}) => {
  const seeded = seedProfile();
  await signIn(page, seeded);

  await page.goto(`${profileHref(seeded)}?tab=repositories`);
  await expect(
    page.getByRole("heading", { name: "Repositories" }),
  ).toBeVisible();
  await expect(page.getByLabel("Search", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Type", { exact: true })).toHaveValue("all");
  await expect(page.getByLabel("Language", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Sort", { exact: true })).toHaveValue(
    "updated-desc",
  );
  await expect(page.getByRole("button", { name: "Filter" })).toBeVisible();
  await expect(
    page.locator(`a[href="${seeded.firstRepositoryHref}"]`).first(),
  ).toBeVisible();
  await expect(page.getByText(/stars/).first()).toBeVisible();
  await expect(page.getByText(/forks/).first()).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-002-phase2-repositories.jpg",
  });
});
