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
  throw new Error("Rust API did not become healthy for repository commits E2E");
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

async function expectNoHorizontalOverflow(page: Page) {
  const dimensions = await page.evaluate(() => ({
    bodyWidth: document.body.scrollWidth,
    viewportWidth: window.innerWidth,
  }));
  expect(dimensions.bodyWidth).toBeLessThanOrEqual(
    dimensions.viewportWidth + 1,
  );
}

async function createRepository(page: Page) {
  const repositoryName = `commit history ${Date.now().toString(36)}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page.getByLabel(/Description/).fill("Commit history Playwright smoke");
  await page
    .getByRole("combobox", { name: /Start with a template/ })
    .selectOption("rust-axum");
  await page.getByRole("button", { name: "Off" }).click();
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));
  return new URL(page.url()).pathname;
}

test.skip(
  !databaseUrl,
  "repository commits E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in commit history renders grouped rows and live links", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryHref = await createRepository(page);

  await page.goto(`${repositoryHref}/commits/main`);
  await expect(
    page.getByRole("heading", { name: "Commit history" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Initial commit/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/commit/`));
  await expect(page.getByRole("link", { name: /checks/ })).toHaveAttribute(
    "href",
    new RegExp(`${repositoryHref}/actions\\?commit=`),
  );
  await expect(
    page.getByRole("link", { name: /Browse repository at/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/tree/`));
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-phase2-default-history.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-final-default-history.jpg",
  });

  await page.goto(
    `${repositoryHref}/commits/main?until=2000-01-01T00%3A00%3A00Z`,
  );
  await expect(
    page.getByRole("heading", { name: "No commits found" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Clear commit filters" }).click();
  await expect(page).toHaveURL(`${repositoryHref}/commits/main`);

  await page.setViewportSize({ width: 390, height: 844 });
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-final-mobile.jpg",
  });
});

test("commit history branch and tag selector reloads refs with filters preserved", async ({
  page,
}) => {
  const seeded = seedSession({ DASHBOARD_E2E_TREE_REFS: "1" });
  await signIn(page, seeded);
  expect(seeded.treeRepositoryHref).toBeTruthy();
  const repositoryHref = seeded.treeRepositoryHref as string;

  await page.goto(
    `${repositoryHref}/commits/main?until=2099-01-01T00%3A00%3A00Z`,
  );
  await expect(
    page.getByRole("heading", { exact: true, name: "Commit history" }),
  ).toBeVisible();

  await page.getByLabel("Switch branches or tags. Current ref main").click();
  await expect(page.getByLabel("Find a branch or tag")).toBeVisible();
  await expect(
    page.getByRole("menuitemradio", { name: /main.*Default.*Selected/ }),
  ).toBeVisible();
  await page.getByLabel("Find a branch or tag").fill("feature");
  await page.getByRole("menuitemradio", { name: /feature\/tree-nav/ }).click();
  await expect(page).toHaveURL(
    new RegExp(
      `${repositoryHref}/commits/feature%2Ftree-nav\\?until=2099-01-01T00%3A00%3A00Z`,
    ),
  );
  await expect(page.getByText("Default history for")).toBeVisible();
  await expect(
    page
      .locator("p")
      .filter({ hasText: "Default history for feature/tree-nav" }),
  ).toBeVisible();

  await page
    .getByLabel("Switch branches or tags. Current ref feature/tree-nav")
    .click();
  await page.getByLabel("Find a branch or tag").fill("v1");
  await page.getByRole("button", { name: /Tags/ }).click();
  await page.getByRole("menuitemradio", { name: /v1\.0\.0/ }).click();
  await expect(page).toHaveURL(
    new RegExp(
      `${repositoryHref}/commits/v1\\.0\\.0\\?until=2099-01-01T00%3A00%3A00Z`,
    ),
  );
  await expect(
    page.locator("p").filter({ hasText: "Default history for v1.0.0" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.getByLabel("Switch branches or tags. Current ref v1.0.0").click();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-phase3-ref-selector.jpg",
  });
});

test("commit history author and date filters are reversible", async ({
  page,
}) => {
  const seeded = seedSession({ DASHBOARD_E2E_TREE_REFS: "1" });
  await signIn(page, seeded);
  expect(seeded.treeRepositoryHref).toBeTruthy();
  const repositoryHref = seeded.treeRepositoryHref as string;

  await page.goto(`${repositoryHref}/commits/main`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Commit history" }),
  ).toBeVisible();

  await page
    .getByLabel("Filter commits by author. Current author All users")
    .click();
  await page.getByLabel("Find an author").fill("dash");
  const authorLink = page
    .getByRole("dialog", { name: "Filter commits by author" })
    .getByRole("link")
    .filter({ hasNotText: "All users" })
    .first();
  const authorHref = await authorLink.getAttribute("href");
  expect(authorHref).toContain("?author=");
  await authorLink.click();
  await expect(page).toHaveURL(
    new RegExp(`${repositoryHref}/commits/main\\?author=`),
  );
  await expect(page.getByText("Active filters")).toBeVisible();

  await page
    .getByLabel("Filter commits by date. Current date All time")
    .click();
  await page.getByLabel("Until date").fill("2099-01-01");
  await page.getByRole("link", { name: "Apply date" }).click();
  await expect(page).toHaveURL(
    new RegExp(
      `${repositoryHref}/commits/main\\?author=.*until=2099-01-01T23%3A59%3A59Z`,
    ),
  );
  await expect(page.getByText("Until 2099-01-01 x")).toBeVisible();

  await page.getByRole("link", { name: "Until 2099-01-01 x" }).click();
  await expect(page).toHaveURL(
    new RegExp(`${repositoryHref}/commits/main\\?author=`),
  );
  await page.getByRole("link", { name: "Clear filters" }).click();
  await expect(page).toHaveURL(`${repositoryHref}/commits/main`);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/commits-001-phase4-filtered-history.jpg",
  });
});
