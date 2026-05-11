import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(skipWithoutTestDb(), "repository Pulse smoke needs a database URL");
test.setTimeout(120_000);

test("repository Pulse renders live overview data and concrete destinations", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await signIn(page, seeded, "owner");

  await page.goto(`${seeded.hrefs.treeRepository}/pulse`);
  await expect(
    page.getByRole("heading", { name: "Repository activity" }),
  ).toBeVisible();
  await expect(page.getByText("Insights").first()).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Pulse Activity summary for the selected period",
    }),
  ).toHaveAttribute("href", /\/pulse$/);
  await expect(
    page.getByRole("link", {
      name: "Contributors Contributor commit activity",
    }),
  ).toHaveAttribute("href", /\/graphs\/contributors$/);
  await page.getByRole("button", { name: "Period: Last week" }).click();
  await expect(page.getByRole("menu", { name: "Pulse period" })).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "Last 24 hours" }),
  ).toHaveAttribute("href", /\/pulse\?period=24h$/);
  await expect(
    page.getByRole("menuitem", { name: "Last month" }),
  ).toHaveAttribute("href", /\/pulse\?period=1m$/);
  await page.getByRole("menuitem", { name: "Last 3 days" }).click();
  await expect(page).toHaveURL(/\/pulse\?period=3d$/);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await expect(page.getByText(/May \d+, 2026 - May \d+, 2026/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);

  await expect(page.getByLabel("Pulse overview metrics")).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Merged pull requests/ }),
  ).toHaveAttribute("href", /\/pulls\?state=merged/);
  await expect(
    page.getByRole("link", { name: /Open pull requests/ }),
  ).toHaveAttribute("href", /\/pulls\?state=open/);
  await expect(
    page.getByRole("link", { name: /Closed issues/ }),
  ).toHaveAttribute("href", /\/issues\?state=closed/);
  await expect(page.getByRole("link", { name: /New issues/ })).toHaveAttribute(
    "href",
    /\/issues\?state=open.*sort=created-desc/,
  );
  await expect(
    page.getByRole("img", { name: "Top committers bar chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Top committers data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /\d+ commits/ }).first(),
  ).toHaveAttribute("href", /\/commits\/main\?.*until=/);
  await expect(
    page.getByRole("link", { name: /pulse-e2e|dashboard/ }).first(),
  ).toHaveAttribute("href", /^\/[^/]+$/);

  const releaseLink = page
    .getByRole("link", { name: /View releases|Release|preview/i })
    .first();
  await expect(releaseLink).toHaveAttribute("href", /\/releases/);
  const pullLink = page.locator('a[href*="/pull"]').first();
  await expect(pullLink).toHaveAttribute("href", /\/pulls|\/pull\//);
  const issueLink = page.locator('a[href*="/issues"]').first();
  await expect(issueLink).toHaveAttribute("href", /\/issues/);

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    animations: "disabled",
    fullPage: true,
    path: screenshotPath(testInfo, "insights-001-final-desktop"),
    timeout: 20_000,
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await expectNoHorizontalOverflow(page);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await page.screenshot({
    animations: "disabled",
    fullPage: true,
    path: screenshotPath(testInfo, "insights-001-final-mobile"),
    timeout: 20_000,
  });
});
