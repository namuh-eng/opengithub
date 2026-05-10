import type { Page } from "@playwright/test";
import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

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
  throw new Error("Rust API did not become healthy for repository Traffic E2E");
}

test.skip(skipWithoutTestDb(), "repository Traffic smoke needs a database URL");
test.setTimeout(90_000);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("repository Traffic renders traffic analytics and concrete links", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await signIn(page, seeded, "owner");

  await page.goto(`${seeded.hrefs.treeRepository}/graphs/traffic`);
  await expect(
    page.getByRole("heading", { name: "Traffic analytics" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Traffic Clone and visitor analytics",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.locator(".chip.active", { hasText: "Last 14 days" }),
  ).toBeVisible();
  await expect(page.locator(".chip", { hasText: "active days" })).toBeVisible();
  await expect(
    page.locator(".chip", { hasText: "Internal traffic excluded" }),
  ).toBeVisible();
  await expect(page.getByLabel("Traffic summary metrics")).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Clones line chart" }),
  ).toBeVisible();
  const clonePoint = page.getByRole("button", {
    name: /Clones .*12 clones, 5 unique cloners/,
  });
  await clonePoint.focus();
  await expect(page.getByText(/12 clones, 5 unique cloners/)).toBeVisible();
  const visitorPoint = page.getByRole("button", {
    name: /Visitors .*48 views, 20 unique visitors/,
  });
  await visitorPoint.hover();
  await expect(page.getByText(/48 views, 20 unique visitors/)).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Clones data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Visitors line chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Visitors data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);
  await expect(
    page.getByRole("link", {
      name: "https://search.opengithub.local/results?q=traffic",
    }),
  ).toHaveAttribute("rel", "noopener noreferrer");
  await expect(
    page.getByRole("link", { name: "Application entrypoint" }),
  ).toHaveAttribute("href", /\/blob\/.*src\/main\.rs$/);
  await expect(
    page.getByRole("link", {
      name: "https://very-long-referrer.example.com/docs/product/analytics/traffic/reports/2026/05/that-keeps-wrapping-in-the-table",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Very long traffic report" }),
  ).toHaveAttribute("href", /very-long-file-name\.md$/);

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "insights-003-final-desktop"),
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Traffic analytics" }),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "insights-003-final-mobile"),
  });
});

test("repository Traffic hides counts from read-only collaborators", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  if (!seeded.hrefs.trafficReadOnlyRepository) {
    throw new Error("dashboard seed did not return read-only traffic fixture");
  }
  await signIn(page, seeded, "outsider");

  await page.goto(`${seeded.hrefs.trafficReadOnlyRepository}/graphs/traffic`);
  const unavailable = page.getByRole("region", {
    name: "Traffic unavailable details",
  });
  await expect(
    unavailable.getByRole("heading", { name: "Traffic unavailable" }),
  ).toBeVisible();
  await expect(
    unavailable.getByText(
      "Repository traffic is available to users with push access.",
    ),
  ).toBeVisible();
  await expect(
    unavailable.getByRole("link", { name: "Back to Code" }),
  ).toHaveAttribute("href", seeded.hrefs.trafficReadOnlyRepository);
  await expect(unavailable.getByText("42")).toHaveCount(0);
  await expect(unavailable.getByText("24")).toHaveCount(0);
  await expect(
    unavailable.getByText("https://search.opengithub.local/results?q=traffic"),
  ).toHaveCount(0);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
});
