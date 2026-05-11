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
  "repository commits E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in commit history renders grouped rows and live links", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed();
  await signIn(page, seeded, "owner");
  const repositoryHref = seeded.hrefs.firstRepository;
  expect(repositoryHref).toBeTruthy();

  await page.goto(`${repositoryHref}/commits/main`);
  await expect(
    page.getByRole("heading", { name: "Commit history" }),
  ).toBeVisible();
  await expect(
    page
      .locator('section[aria-label="Grouped commits"] a[href*="/commit/"]')
      .first(),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/commit/`));
  await expect(
    page.getByRole("link", { name: /checks|No checks/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/actions\\?commit=`));
  await expect(
    page.getByRole("link", { name: /Browse repository at/ }),
  ).toHaveAttribute("href", new RegExp(`${repositoryHref}/tree/`));
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "commits-001-phase2-default-history"),
  });
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "commits-001-final-default-history"),
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
    path: screenshotPath(testInfo, "commits-001-final-mobile"),
  });
});

test("commit history branch and tag selector reloads refs with filters preserved", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await signIn(page, seeded, "owner");
  const repositoryHref = seeded.hrefs.treeRepository;
  expect(repositoryHref).toBeTruthy();

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
    path: screenshotPath(testInfo, "commits-001-phase3-ref-selector"),
  });
});

test("commit history author and date filters are reversible", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await signIn(page, seeded, "owner");
  const repositoryHref = seeded.hrefs.treeRepository;
  expect(repositoryHref).toBeTruthy();

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
    path: screenshotPath(testInfo, "commits-001-phase4-filtered-history"),
  });
});
