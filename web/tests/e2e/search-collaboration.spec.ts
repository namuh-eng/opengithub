import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

function seedSession(marker: string): SeededSession {
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
        SEARCH_E2E_MARKER: marker,
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
  "collaboration search E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("issue search sort, saved search, compact view, and navigation are concrete", async ({
  page,
}) => {
  const marker = `search004p4${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(`/search?q=${marker}&type=issues`);
  await expect(
    page.getByRole("heading", { name: "Issues search" }),
  ).toBeVisible();
  await expect(page.getByText(/1 issues results/)).toBeVisible();

  await page.getByText("Sort by: Best match").click();
  await page.getByRole("link", { name: "Most commented" }).click();
  await expect(page).toHaveURL(/sort=most_commented/);
  await expect(page.getByText("Sort by: Most commented")).toBeVisible();

  await page.getByRole("link", { name: "Compact" }).click();
  await expect(page).toHaveURL(/view=compact/);
  await expect(page.getByRole("link", { name: "Compact" })).toHaveAttribute(
    "aria-current",
    "true",
  );

  await page.getByRole("link", { name: "Save" }).click();
  await expect(page).toHaveURL(/saved=1/);
  const savedName = `Issue search ${Date.now()}`;
  await page.getByLabel("Saved search name").fill(savedName);
  await page.getByRole("button", { name: "Create saved search" }).click();
  await expect(page.getByText(`Saved "${savedName}".`)).toBeVisible();

  const issueResult = page.getByRole("link", {
    name: new RegExp(`Investigate ${marker} issue search`),
  });
  await expect(issueResult).toHaveAttribute("href", /\/issues\/1$/);
  await issueResult.click();
  await expect(page).toHaveURL(/\/issues\/1$/);

  await expectNoDeadControls(page);
});

test("pull request search preserves sort/view and empty-state recovery is actionable", async ({
  page,
}) => {
  const marker = `search004p4pr${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(
    `/search?q=${marker}&type=pull_requests&sort=least_commented&view=compact`,
  );
  await expect(
    page.getByRole("heading", { name: "Pull requests search" }),
  ).toBeVisible();
  await expect(page.getByText(/1 pull requests results/)).toBeVisible();
  await expect(page.getByText("Sort by: Least commented")).toBeVisible();
  await expect(page.getByRole("link", { name: "Compact" })).toHaveAttribute(
    "aria-current",
    "true",
  );

  const pullResult = page.getByRole("link", {
    name: new RegExp(`Review ${marker} pull search`),
  });
  await expect(pullResult).toHaveAttribute("href", /\/pull\/2$/);

  await page.goto(
    `/search?q=${marker}%20label:missing&type=pull_requests&sort=least_commented&view=compact`,
  );
  await expect(page.getByText(/No pull requests matched/)).toBeVisible();
  await page.getByRole("link", { name: /Remove label:missing/ }).click();
  await expect(page).not.toHaveURL(/label%3Amissing|label:missing/);
  await expect(page).toHaveURL(/sort=least_commented/);
  await expect(page).toHaveURL(/view=compact/);
  await expect(page.getByText(/1 pull requests results/)).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-004-phase4-sort-save-pagination.jpg",
  });
});
