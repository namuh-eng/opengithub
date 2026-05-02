import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededProfile = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
  secondRepositoryHref: string;
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

  const repositoryName = seeded.firstRepositoryHref.split("/").at(-1) ?? "";
  await page.getByLabel("Search", { exact: true }).fill(repositoryName);
  await page.getByLabel("Sort", { exact: true }).selectOption("stars-desc");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(/tab=repositories/);
  await expect(page).toHaveURL(new RegExp(`q=${repositoryName}`));
  await expect(page).toHaveURL(/sort=stars-desc/);
  await expect(
    page.getByRole("link", { name: `Search: ${repositoryName} x` }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Sort: Stars x" })).toBeVisible();

  const typeOptions = await page
    .getByLabel("Type", { exact: true })
    .locator("option")
    .evaluateAll((options) =>
      options
        .map((option) => (option as HTMLOptionElement).value)
        .filter((value) => value && value !== "all"),
    );
  if (typeOptions.length > 0) {
    await page.getByLabel("Type", { exact: true }).selectOption(typeOptions[0]);
    await page.getByRole("button", { name: "Filter" }).click();
    await expect(page).toHaveURL(new RegExp(`type=${typeOptions[0]}`));
  }

  await page.getByRole("link", { name: "Sort: Stars x" }).click();
  await expect(page).not.toHaveURL(/sort=stars-desc/);
  await page.getByRole("link", { name: "Clear filters" }).first().click();
  await expect(page).toHaveURL(
    new RegExp(`${profileHref(seeded)}\\?tab=repositories$`),
  );
  await expect(
    page.locator(`a[href="${seeded.firstRepositoryHref}"]`).first(),
  ).toBeVisible();

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-002-phase3-repository-filters.jpg",
  });

  await page.locator(`a[href="${seeded.firstRepositoryHref}"]`).first().click();
  await expect(page).toHaveURL(new RegExp(`${seeded.firstRepositoryHref}$`));
});

test("profile stars tab renders starred rows and stars filters", async ({
  page,
}) => {
  const seeded = seedProfile();
  await signIn(page, seeded);

  await page.goto(`${profileHref(seeded)}?tab=stars`);
  await expect(
    page.getByRole("heading", { name: "Starred repositories" }),
  ).toBeVisible();
  await expect(page.getByLabel("Search", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Type", { exact: true })).toHaveCount(0);
  await expect(page.getByLabel("Language", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Sort", { exact: true })).toHaveValue(
    "recently-starred",
  );
  await expect(page.getByText(/Starred/).first()).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  const starredRepositoryName =
    seeded.secondRepositoryHref.split("/").at(-1) ?? "";
  await page.getByLabel("Search", { exact: true }).fill(starredRepositoryName);
  await page
    .getByLabel("Sort", { exact: true })
    .selectOption("recently-active");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(/tab=stars/);
  await expect(page).toHaveURL(new RegExp(`q=${starredRepositoryName}`));
  await expect(page).toHaveURL(/sort=recently-active/);
  await expect(
    page.getByRole("link", { name: `Search: ${starredRepositoryName} x` }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Sort: Recently active x" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Sort: Recently active x" }).click();
  await expect(page).not.toHaveURL(/sort=recently-active/);
  await page.getByRole("link", { name: "Clear filters" }).first().click();
  await expect(page).toHaveURL(
    new RegExp(`${profileHref(seeded)}\\?tab=stars$`),
  );
  await expect(
    page.locator(`a[href="${seeded.secondRepositoryHref}"]`).first(),
  ).toBeVisible();

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-002-phase4-stars.jpg",
  });

  await page
    .locator(`a[href="${seeded.secondRepositoryHref}"]`)
    .first()
    .click();
  await expect(page).toHaveURL(new RegExp(`${seeded.secondRepositoryHref}$`));
});

test("anonymous mobile profile repository tabs stay responsive and actionable", async ({
  page,
}) => {
  const seeded = seedProfile();
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto(`${profileHref(seeded)}?tab=repositories`);
  await expect(
    page.getByRole("heading", { name: "Repositories" }),
  ).toBeVisible();
  await expect(
    page.locator(`a[href="${seeded.firstRepositoryHref}"]`).first(),
  ).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.getByLabel("Search", { exact: true }).fill("no-results-final");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(
    page.getByText("No repositories matched these filters."),
  ).toBeVisible();
  await page.getByRole("link", { name: "Clear filters" }).first().click();
  await expect(page).toHaveURL(
    new RegExp(`${profileHref(seeded)}\\?tab=repositories$`),
  );
  let overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-002-final-mobile-repositories.jpg",
  });

  await page.goto(`${profileHref(seeded)}?tab=stars`);
  await expect(
    page.getByRole("heading", { name: "Starred repositories" }),
  ).toBeVisible();
  await expect(page.getByLabel("Search", { exact: true })).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page
    .getByLabel("Sort", { exact: true })
    .selectOption("recently-active");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(/tab=stars/);
  await expect(page).toHaveURL(/sort=recently-active/);
  await page.getByRole("link", { name: "Sort: Recently active x" }).click();
  await expect(page).not.toHaveURL(/sort=recently-active/);
  overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-002-final-mobile-stars.jpg",
  });
});
