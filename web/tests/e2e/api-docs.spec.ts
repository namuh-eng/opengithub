import { expect, test } from "@playwright/test";

async function expectNoDeadControls(page: import("@playwright/test").Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test("api docs expose implemented REST surfaces and working examples", async ({
  page,
}) => {
  await page.goto("/docs/api");

  await expect(
    page.getByRole("heading", {
      name: "Build against implemented opengithub APIs",
    }),
  ).toBeVisible();
  await expect(page.getByText("GET").first()).toBeVisible();
  await expect(
    page.locator("code").filter({ hasText: "/api/user" }),
  ).toBeVisible();
  await expect(
    page.getByText("/api/repos/{owner}/{repo}/issues"),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: /^\/api\/repos\/\{owner\}\/\{repo\}\/pulls$/,
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/pulls/{number}.diff",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText:
        "/api/repos/{owner}/{repo}/actions/dashboard?q=ci&status=success&page=1&pageSize=30",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/runs?page=1&pageSize=30",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: /^\/api\/repos\/\{owner\}\/\{repo\}\/actions\/runs\/\{run_id\}$/,
    }),
  ).toHaveCount(2);
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/detail",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/rerun",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/cancel",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText:
        /^\/api\/repos\/\{owner\}\/\{repo\}\/actions\/runs\/\{run_id\}\/logs$/,
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText:
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}/detail?q=error&match=1&timestamps=true&raw=false",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/log-preferences",
    }),
  ).toBeVisible();
  await expect(
    page.locator("code").filter({
      hasText: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs/archive",
    }),
  ).toBeVisible();
  await expect(
    page
      .locator("code")
      .filter({ hasText: "/api/search?q=router&type=code&page=1&pageSize=30" }),
  ).toBeVisible();
  await expect(page.locator("article")).not.toContainText("api.github.com");

  const firstExample = page.getByText("Request and response examples").first();
  await firstExample.click();
  await expect(page.getByText('"login": "mona"')).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Git docs" })).toBeVisible();
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/api-001-docs.jpg",
  });
});

test("api docs mobile layout does not overflow horizontally", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/docs/api");

  await expect(
    page.getByRole("heading", {
      name: "Build against implemented opengithub APIs",
    }),
  ).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    innerWidth: window.innerWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.innerWidth);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/api-001-docs-mobile.jpg",
  });
});
