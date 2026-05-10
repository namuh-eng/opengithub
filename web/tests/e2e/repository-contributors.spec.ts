import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(
  skipWithoutTestDb(),
  "repository Contributors smoke needs a database URL",
);
test.setTimeout(90_000);

test("repository Contributors renders default analytics and concrete drilldowns", async ({
  page,
  seed,
  seedContributorFileChanges,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await seedContributorFileChanges(seeded.hrefs.treeRepository);
  await signIn(page, seeded, "owner");

  await page.goto(`${seeded.hrefs.treeRepository}/graphs/contributors`);
  await expect(
    page.getByRole("heading", { name: "Contributor analytics" }),
  ).toBeVisible();
  await expect(page.getByText(/Default branch scope:/)).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Contributors Contributor commit activity",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("button", { name: "Period: Last week" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: "Repository commits over time chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "View as data table" }),
  ).toHaveAttribute("aria-expanded", "false");
  await page.getByRole("button", { name: "View as data table" }).click();
  await expect(
    page.getByRole("table", { name: "Repository contributors data table" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "View as data table" }).first(),
  ).toHaveAttribute("href", "#contributors-data-table-panel");
  await expect(
    page.getByRole("link", { name: "Commit history" }),
  ).toHaveAttribute("href", /\/commits\/main$/);
  await expect(
    page.getByRole("link", { name: /\d+ commits/ }).first(),
  ).toHaveAttribute("href", /\/commits\/main\?.*author=/);
  await expect(
    page.getByRole("link", { name: /pulse-e2e|dashboard/ }).first(),
  ).toHaveAttribute("href", /^\/[^/]+$/);

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "insights-002-final-directory"),
  });

  await page.getByRole("button", { name: "Period: Last week" }).click();
  await expect(
    page.getByRole("menu", { name: "Contributors period" }),
  ).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "Last 24 hours" }),
  ).toHaveAttribute("href", /\/graphs\/contributors\?period=24h$/);
  await expect(
    page.getByRole("menuitem", { name: "Last month" }),
  ).toHaveAttribute("href", /\/graphs\/contributors\?period=1m$/);
  await page.getByRole("menuitem", { name: "Last 3 days" }).click();
  await expect(page).toHaveURL(/\/graphs\/contributors\?period=3d$/);
  await expect(
    page.getByRole("button", { name: "Period: Last 3 days" }),
  ).toBeVisible();
  await expect(page.getByRole("slider", { name: "Start week" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Apply" })).toHaveAttribute(
    "href",
    /\/graphs\/contributors\?period=3d(&start=.*&end=.*)?$/,
  );
  await expect(
    page.getByRole("link", { name: "Download CSV" }),
  ).toHaveAttribute("download", "repository-contributors.csv");
  await page.getByRole("button", { name: "Copy CSV" }).click();
  await expect(page.getByText("CSV copied")).toBeVisible();
  await page.goto(`${seeded.hrefs.treeRepository}/pulse`);
  await page
    .getByRole("link", {
      name: "Contributors Contributor commit activity",
    })
    .click();
  await expect(page).toHaveURL(/\/graphs\/contributors$/);
  await expect(
    page.getByRole("heading", { name: "Contributor analytics" }),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 900 });
  await expect(
    page.getByRole("heading", { name: "Contributor analytics" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /\d+ commits/ }).first(),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "insights-002-final-mobile"),
  });
});
