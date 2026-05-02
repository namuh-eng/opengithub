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
  "profile overview E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("public profile overview renders data, tabs, and pinned navigation", async ({
  page,
}) => {
  const seeded = seedProfile();
  await signIn(page, seeded);

  await page.goto(profileHref(seeded));
  await expect(
    page.getByRole("heading", { name: "Dashboard Tester" }),
  ).toBeVisible();
  await expect(page.getByText(/@dash-/)).toBeVisible();
  await expect(page.getByRole("heading", { name: "README" })).toBeVisible();
  await expect(page.getByText(/Seeded profile overview/)).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Profile sections" }),
  ).toBeVisible();

  const pinnedRepository = page.getByRole("link", {
    name: /alpha-/,
  });
  await expect(pinnedRepository).toHaveAttribute(
    "href",
    seeded.firstRepositoryHref,
  );
  await pinnedRepository.click();
  await expect(page).toHaveURL(new RegExp(`${seeded.firstRepositoryHref}$`));

  await page.goto(`${profileHref(seeded)}?tab=repositories`);
  await expect(page.getByRole("link", { name: /Stars/ })).toHaveAttribute(
    "href",
    `${profileHref(seeded)}?tab=stars`,
  );
  await page.getByRole("link", { name: /Stars/ }).click();
  await expect(page).toHaveURL(
    new RegExp(`${profileHref(seeded)}\\?tab=stars$`),
  );
  await expect(
    page.getByRole("heading", { name: /Stars for dash-/ }),
  ).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-001-phase2-overview.jpg",
  });
});
