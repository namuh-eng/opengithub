import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
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
        SEARCH_E2E_MARKER: "searchmodal",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard) {
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

async function expectHeaderControlsWork(page: Page) {
  const header = page.locator(".app-shell-header").first();
  await expect(
    header.getByRole("link", { name: "opengithub dashboard" }),
  ).toHaveAttribute("href", "/dashboard");
  await expect(
    header.getByRole("button", { name: "Global menu" }),
  ).toBeVisible();
  await expect(
    header.getByRole("button", { name: "Create new" }),
  ).toBeVisible();
  await expect(
    header.getByRole("button", { name: "Open user menu" }),
  ).toBeVisible();
  await expect(
    header.getByRole("link", { name: /notifications/i }),
  ).toHaveAttribute("href", "/notifications");
  await expectNoDeadControls(page);
}

test.skip(
  !databaseUrl,
  "app shell E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in desktop header menus, links, search, and sign-out work", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/dashboard");

  await expect(
    page.getByRole("link", { name: "opengithub dashboard" }),
  ).toHaveAttribute("href", "/dashboard");
  await expect(
    page.getByRole("link", { name: "Pull requests" }),
  ).toHaveAttribute("href", "/pulls");
  await expect(page.getByRole("link", { name: "Issues" })).toHaveAttribute(
    "href",
    "/issues",
  );
  await expectNoDeadControls(page);

  await page.getByRole("button", { name: "Global menu" }).click();
  await expect(
    page.getByRole("menuitem", { name: "Dashboard" }),
  ).toHaveAttribute("href", "/dashboard");
  const recentRepository = page.getByRole("menuitem", {
    name: seeded.firstRepositoryHref.slice(1),
  });
  await expect(recentRepository).toHaveAttribute(
    "href",
    seeded.firstRepositoryHref,
  );

  await page.getByRole("button", { name: "Create new" }).click();
  await expect(
    page.getByRole("menuitem", { name: "New repository" }),
  ).toHaveAttribute("href", "/new");
  await expect(
    page.getByRole("menuitem", { name: "Import repository" }),
  ).toHaveAttribute("href", "/new/import");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-create-menu.jpg",
  });

  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  const searchDialog = page.getByRole("dialog", { name: "Search" });
  await expect(searchDialog).toBeVisible();
  await expect(
    searchDialog.getByRole("combobox", { name: "Search opengithub" }),
  ).toBeFocused();
  await expect(searchDialog.getByRole("listbox")).toBeVisible();
  await expect(
    searchDialog.getByRole("link", { name: "Syntax tips" }),
  ).toHaveAttribute("href", "/docs/api#search");
  await expect(
    searchDialog.getByRole("link", { name: "Feedback" }),
  ).toHaveAttribute("href", "/issues/new?title=Search%20feedback");
  await expect(
    searchDialog.getByRole("option", { name: /Repositories/ }).first(),
  ).toBeVisible();
  await expect(
    searchDialog.getByRole("option", { name: /searchmodal/ }).first(),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-002-phase2-command-modal.jpg",
  });
  await page.keyboard.press("Escape");
  await expect(searchDialog).toHaveCount(0);

  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  await searchDialog
    .getByRole("combobox", { name: "Search opengithub" })
    .fill("rust");
  await searchDialog.getByRole("link", { exact: true, name: "Search" }).click();
  await expect(page).toHaveURL(/\/search\?q=rust&type=repositories$/);
  await expect(
    page.getByRole("heading", { name: "Search opengithub" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Open user menu" }).click();
  await expect(
    page.getByRole("menuitem", { name: "Your profile" }),
  ).toHaveAttribute("href", "/settings/profile");
  await expect(
    page.getByRole("menuitem", { name: "Developer settings" }),
  ).toHaveAttribute("href", "/settings/tokens");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-avatar-menu.jpg",
  });

  await page.getByRole("menuitem", { name: "Sign out" }).click();
  await expect(page).toHaveURL("http://localhost:3015/");
});

test("global search modal autocompletes qualifiers and follows direct jumps", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/dashboard");
  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  const searchDialog = page.getByRole("dialog", { name: "Search" });
  const combobox = searchDialog.getByRole("combobox", {
    name: "Search opengithub",
  });
  await expect(combobox).toBeFocused();

  await combobox.fill("language:ru");
  await searchDialog.getByRole("option", { name: /language:rust/ }).click();
  await expect(combobox).toHaveValue("language:rust ");

  await searchDialog.getByRole("button", { name: "path:src/" }).click();
  await expect(combobox).toHaveValue("language:rust path:src/ ");

  await combobox.fill("searchmodal");
  await searchDialog
    .getByRole("option", { name: /searchmodal/ })
    .first()
    .click();
  await expect(page).toHaveURL(/\/[^/]+\/search-[^/?#]+$/);

  await page.goto("/dashboard");
  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  await combobox.fill("searchmodal");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/search\?q=searchmodal&type=repositories$/);

  await page.goto("/dashboard");
  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  await combobox.fill("language:ru");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-002-phase3-autocomplete-jumps.jpg",
  });
});

test("signed-in mobile drawer exposes navigation and responsive frames", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto("/dashboard");
  await expect(page.locator("[data-app-shell-frame='centered']")).toBeVisible();
  await expectHeaderControlsWork(page);

  await page.getByRole("button", { name: "Global menu" }).click();
  const drawer = page.getByRole("dialog", { name: "Global menu" });
  await expect(drawer).toBeVisible();
  await expect(
    drawer.getByRole("link", { name: /Pull requests/ }),
  ).toHaveAttribute("href", "/pulls");
  await expect(
    drawer.getByRole("link", { name: seeded.firstRepositoryHref.slice(1) }),
  ).toHaveAttribute("href", seeded.firstRepositoryHref);
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();

  await page.goto(seeded.firstRepositoryHref);
  await expect(
    page.locator("[data-app-shell-frame='repository']"),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-mobile-drawer.jpg",
  });
});

test("signed-in shell is stable across primary destinations", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const seededRepositoryName =
    seeded.firstRepositoryHref.split("/").at(-1) ?? "";

  const destinations = [
    { path: "/dashboard", heading: "Dashboard" },
    { path: seeded.firstRepositoryHref, heading: seededRepositoryName },
    { path: "/new", heading: "Create a new repository" },
    { path: "/notifications", heading: "Notifications" },
    { path: "/issues", heading: "Issues" },
    { path: "/pulls", heading: "Pull requests" },
    { path: "/settings/profile", heading: "Profile" },
  ] as const;

  for (const destination of destinations) {
    await page.goto(destination.path);
    await expect(
      page.getByRole("heading", { name: destination.heading }).first(),
    ).toBeVisible();
    await expectHeaderControlsWork(page);
    const horizontalOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth > window.innerWidth,
    );
    expect(horizontalOverflow).toBe(false);
  }

  await page.goto("/dashboard");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-dashboard-frame.jpg",
  });

  await page.goto(seeded.firstRepositoryHref);
  await expect(
    page.locator("[data-app-shell-frame='repository']"),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-repository-frame.jpg",
  });

  await page.getByRole("button", { name: "Global menu" }).click();
  await expect(page.getByRole("menu")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/layout-001-final-desktop-header.jpg",
  });
});
