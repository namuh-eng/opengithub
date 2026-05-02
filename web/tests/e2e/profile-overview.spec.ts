import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededProfile = {
  cookieName: string;
  cookieValue: string;
  profileActionCookieValue: string;
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
  await signInWithValue(page, seeded, seeded.cookieValue);
}

async function signInWithValue(
  page: Page,
  seeded: SeededProfile,
  cookieValue: string,
) {
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: cookieValue,
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
    page.getByRole("heading", {
      name: new RegExp(`contributions in ${new Date().getFullYear()}`),
    }),
  ).toBeVisible();
  await expect(page.getByLabel(/contributions on/).first()).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Profile sections" }),
  ).toBeVisible();

  const pinnedRepository = page.locator(
    `a[href="${seeded.firstRepositoryHref}"]`,
  );
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
  await page.goto(
    `${profileHref(seeded)}?year=${new Date().getFullYear() - 1}`,
  );
  await expect(
    page.getByRole("link", { name: String(new Date().getFullYear() - 1) }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByText(/No public contributions are visible/),
  ).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(profileHref(seeded));
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-001-phase4-contributions.jpg",
  });
});

test("profile actions follow, login-gate, block, and report through real routes", async ({
  page,
}) => {
  const seeded = seedProfile();

  await page.goto(profileHref(seeded));
  await page.getByRole("button", { name: "Follow" }).click();
  const loginDialog = page.getByRole("dialog", {
    name: new RegExp(
      `Continue to interact with @${profileHref(seeded).slice(1)}`,
    ),
  });
  await expect(loginDialog).toBeVisible();
  await expect(
    loginDialog.getByRole("link", { name: "Sign in" }),
  ).toHaveAttribute("href", /\/login\?next=/);

  await signInWithValue(page, seeded, seeded.profileActionCookieValue);
  await page.goto(profileHref(seeded));
  await page.getByRole("button", { name: "Follow" }).click();
  await expect(page.getByRole("button", { name: "Following" })).toBeVisible();
  await expect(page.getByText("Now following this profile.")).toBeVisible();

  await page.getByRole("button", { name: "More" }).click();
  await page.getByRole("menuitem", { name: "Report profile" }).click();
  await expect(
    page.getByRole("dialog", { name: /Tell us what is wrong/ }),
  ).toBeVisible();
  await page.getByLabel("Details").fill("Seeded browser smoke report.");
  await page.getByRole("button", { name: "Submit report" }).click();
  await expect(page.getByText("Report submitted for review.")).toBeVisible();

  await page.getByRole("button", { name: "More" }).click();
  await page.getByRole("menuitem", { name: "Block profile" }).click();
  await expect(page.getByRole("dialog", { name: /Block @/ })).toBeVisible();
  await page.getByRole("button", { name: "Block" }).click();
  await expect(page.getByText("Profile blocked.")).toBeVisible();
  await page.getByRole("button", { name: "More" }).click();
  await expect(page.getByRole("menuitem", { name: "Blocked" })).toBeDisabled();

  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/profiles-001-phase3-actions.jpg",
  });
});
