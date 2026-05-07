import type { Page } from "@playwright/test";
import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

async function boundingBoxFor(locator: ReturnType<Page["locator"]>) {
  const box = await locator.boundingBox();
  expect(box).not.toBeNull();

  if (!box) {
    throw new Error("expected element to have a bounding box");
  }

  return box;
}

test.skip(
  skipWithoutTestDb(),
  "dashboard signed-in E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in dashboard filters top repositories and navigates rows", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded);

  await page.goto("/dashboard");

  await expect(
    page.getByRole("heading", { name: "Top repositories" }),
  ).toBeVisible();
  const newRepositoryLink = page.getByRole("link", {
    exact: true,
    name: "New",
  });
  await expect(newRepositoryLink).toHaveAttribute("href", "/new");
  await expect(page.getByText("Rust")).toBeVisible();
  await expect(page.getByText("TypeScript")).toBeVisible();
  await expect(
    page.getByRole("heading", { exact: true, name: "Dashboard feed" }),
  ).toBeVisible();
  const dashboardFeed = page.locator(
    'section[aria-labelledby="dashboard-feed-heading"]',
  );
  await expect(
    dashboardFeed.getByRole("link", {
      name: "Asked for help reviewing dashboard feed",
    }),
  ).toBeVisible();
  await expect(
    dashboardFeed.getByRole("tab", { name: "Following" }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(
    dashboardFeed.getByRole("tab", { name: "For you" }),
  ).toHaveAttribute("href", "/dashboard?feedTab=for_you");

  const topRepositories = page.getByRole("complementary", {
    name: "Top repositories",
  });
  await page.getByLabel("Find a repository").fill("infra");
  const filteredRepository = topRepositories.getByRole("link", {
    name: /infra-/,
  });
  await expect(filteredRepository).toBeVisible();
  await expect(
    topRepositories.getByRole("link", { name: /alpha-/ }),
  ).toHaveCount(0);

  await filteredRepository.click();
  await expect(page).toHaveURL(new RegExp(`${seeded.hrefs.secondRepository}$`));

  await page.goto("/dashboard");
  await dashboardFeed
    .getByRole("link", { name: "Asked for help reviewing dashboard feed" })
    .click();
  await expect(page).toHaveURL(/\/pull\/\d+$/);
  await expect(
    page.getByRole("heading", { name: /Pull request #\d+/ }),
  ).toBeVisible();

  await page.goto("/dashboard");
  await dashboardFeed.getByRole("tab", { name: "For you" }).click();
  await expect(page).toHaveURL(/feedTab=for_you/);
  await expect(
    dashboardFeed.getByRole("link", {
      name: "Published infrastructure preview",
    }),
  ).toBeVisible();

  await dashboardFeed.locator("summary").click();
  await dashboardFeed.getByLabel("Releases").check();
  await dashboardFeed.getByRole("button", { name: "Apply" }).click();
  await expect(page).toHaveURL(/eventType=release/);
  await expect(
    dashboardFeed.getByRole("link", {
      name: "Published infrastructure preview",
    }),
  ).toBeVisible();
  await expect(
    dashboardFeed.getByRole("link", { name: "Pushed dashboard activity feed" }),
  ).toHaveCount(0);

  await dashboardFeed.locator("summary").click();
  await dashboardFeed.getByRole("link", { name: "Clear filters" }).click();
  await expect(page).toHaveURL(/feedTab=for_you/);
  await expect(page).not.toHaveURL(/eventType=/);

  await page.goto("/dashboard");
  await newRepositoryLink.click();
  await expect(page).toHaveURL(/\/new$/);
});

test("signed-in dashboard feed filters support keyboard use and empty states", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded);

  await page.goto("/dashboard?feedTab=for_you");
  const dashboardFeed = page.locator(
    'section[aria-labelledby="dashboard-feed-heading"]',
  );
  const filterSummary = dashboardFeed.locator("summary");
  const filterDetails = dashboardFeed.locator("details");
  const filterForm = dashboardFeed.locator("form");

  await filterSummary.focus();
  await page.keyboard.press("Enter");
  await expect(filterDetails).toHaveAttribute("open", "");
  await expect(filterForm).toBeVisible();

  await dashboardFeed.getByLabel("Follows").check();
  await dashboardFeed.getByRole("button", { name: "Apply" }).press("Enter");
  await expect(page).toHaveURL(/feedTab=for_you/);
  await expect(page).toHaveURL(/eventType=follow/);
  await expect(
    dashboardFeed.getByText(
      "No dashboard feed events match the current filters.",
    ),
  ).toBeVisible();
  await expect(
    dashboardFeed.getByRole("link", { name: "Clear filters" }),
  ).toHaveAttribute("href", "/dashboard?feedTab=for_you");
  await expect(
    dashboardFeed.getByRole("link", { name: "Create repository" }),
  ).toHaveAttribute("href", "/new");
  await expect(
    dashboardFeed.getByRole("link", { name: "Explore repositories" }),
  ).toHaveAttribute("href", "/explore");
  await expectNoDeadControls(page);
});

test("signed-in dashboard has no dead controls on empty and non-empty states", async ({
  page,
  seed,
  signIn,
}) => {
  const emptySeed = await seed({ scenes: ["empty"] });
  await signIn(page, emptySeed);
  await page.goto("/dashboard");

  await expect(
    page.getByText("You do not have any repositories yet."),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Create repository" }).first(),
  ).toHaveAttribute("href", "/new");
  await expect(
    page.getByRole("link", { name: "Import repository" }).first(),
  ).toHaveAttribute("href", "/new/import");
  await expect(
    page.getByRole("link", { name: "Read setup guide" }).first(),
  ).toHaveAttribute("href", "/docs/get-started");
  await expectNoDeadControls(page);

  const seeded = await seed();
  await page.context().clearCookies();
  await signIn(page, seeded);
  await page.goto("/dashboard");

  await expect(
    page.getByRole("heading", { exact: true, name: "Dashboard feed" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
});

test("signed-in dashboard stacks without horizontal scroll on mobile", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded);

  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto("/dashboard");

  await expect(
    page.getByRole("heading", { name: "Top repositories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { exact: true, name: "Dashboard feed" }),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);

  const sidebar = await boundingBoxFor(
    page.getByRole("complementary", { name: "Top repositories" }),
  );
  const feed = await boundingBoxFor(
    page.locator('section[aria-labelledby="dashboard-feed-heading"]'),
  );
  expect(feed.y).toBeGreaterThan(sidebar.y + sidebar.height - 1);

  await page.getByLabel("Find a repository").fill("infra");
  await expect(
    page
      .getByRole("complementary", { name: "Top repositories" })
      .getByRole("link", { name: /infra-/ }),
  ).toBeVisible();

  const dashboardFeed = page.locator(
    'section[aria-labelledby="dashboard-feed-heading"]',
  );
  await dashboardFeed.locator("summary").click();
  await expect(dashboardFeed.locator("form")).toBeVisible();
  await expectNoHorizontalOverflow(page);
});

test("signed-in dashboard keeps the sidebar and feed aligned on desktop", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/dashboard");

  await expectNoHorizontalOverflow(page);
  const sidebar = await boundingBoxFor(
    page.getByRole("complementary", { name: "Top repositories" }),
  );
  const feed = await boundingBoxFor(
    page.locator('section[aria-labelledby="dashboard-feed-heading"]'),
  );
  const feedTabs = await boundingBoxFor(page.getByRole("tablist"));
  const filterSummary = await boundingBoxFor(
    page
      .locator('section[aria-labelledby="dashboard-feed-heading"]')
      .locator("summary"),
  );
  expect(sidebar.width).toBeGreaterThanOrEqual(290);
  expect(sidebar.width).toBeLessThanOrEqual(306);
  expect(feed.x).toBeGreaterThan(sidebar.x + sidebar.width);
  expect(feed.width).toBeLessThanOrEqual(720);
  expect(filterSummary.x).toBeGreaterThan(feedTabs.x + feedTabs.width);

  await page
    .locator('section[aria-labelledby="dashboard-feed-heading"]')
    .locator("summary")
    .click();
  const filterForm = await boundingBoxFor(
    page.locator('section[aria-labelledby="dashboard-feed-heading"] form'),
  );
  expect(filterForm.x + filterForm.width).toBeLessThanOrEqual(
    feed.x + feed.width + 1,
  );
});
