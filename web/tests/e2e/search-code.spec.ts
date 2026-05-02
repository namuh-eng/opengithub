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
  "code search E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("code search groups snippets, expands hidden matches, and opens line anchors", async ({
  page,
}) => {
  const marker = `phase3code${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(`/search?q=${marker}&type=code`);
  await expect(
    page.getByRole("heading", { name: "Search indexed code" }),
  ).toBeVisible();
  await expect(page.getByText(/1 code results/)).toBeVisible();
  await expect(page.getByText("5 matches")).toBeVisible();
  await expect(
    page.getByRole("link", { name: /src\/search_phase_three\.rs/ }),
  ).toHaveAttribute("href", /#L1$/);
  const resultCard = page
    .locator("article")
    .filter({ hasText: "src/search_phase_three.rs" });
  const lineOne = resultCard.getByRole("link", { exact: true, name: "1" });
  const lineThree = resultCard.getByRole("link", { exact: true, name: "3" });
  const lineFive = resultCard.getByRole("link", { exact: true, name: "5" });
  await expect(lineOne).toHaveAttribute(
    "href",
    /\/blob\/main\/src\/search_phase_three\.rs#L1$/,
  );
  await expect(lineThree).toHaveAttribute(
    "href",
    /\/blob\/main\/src\/search_phase_three\.rs#L3$/,
  );
  await expect(lineFive).toHaveCount(0);

  await page.getByRole("button", { name: "Show 2 more matches" }).click();
  await expect(lineFive).toHaveAttribute(
    "href",
    /\/blob\/main\/src\/search_phase_three\.rs#L5$/,
  );

  await page.getByRole("button", { name: "Collapse" }).click();
  await expect(page.getByText(/Snippets hidden/)).toBeVisible();
  await expect(lineOne).toHaveCount(0);
  await page.getByRole("button", { name: "Expand" }).click();
  await expect(lineOne).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-003-phase3-snippet-expansion.jpg",
  });

  await lineThree.click();
  await expect(page).toHaveURL(/\/blob\/main\/src\/search_phase_three\.rs#L3$/);
  await expect(page.locator("#L3")).toBeVisible();
});
