import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(skipWithoutTestDb(), "organization teams E2E needs a test database");

test("organization teams directory supports owner/member views, filters, and navigation", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  test.setTimeout(90_000);
  const seeded = await seed({ scenes: ["orgProfile"] });
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.organizationProfile}/teams`);
  await expect(page.getByRole("heading", { name: "Teams" })).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await expect(
    page.getByRole("link", { name: /Open Platform Maintainers/ }),
  ).toHaveAttribute("href", /\/orgs\/org-profile-[^/]+\/teams\/platform-/);
  await expect(page.getByText("Security Response")).toBeVisible();
  await expect(page.getByRole("link", { name: "Secret" })).toBeVisible();
  await expect(page.getByText("Mention notifications").first()).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-populated-directory"),
  });

  await page.getByLabel("Search organization teams").fill("frontend");
  await page.getByLabel("Filter team visibility").selectOption("visible");
  await page.getByRole("button", { name: "Filter" }).click();
  await expect(page).toHaveURL(
    /\/orgs\/org-profile-[^/]+\/teams\?q=frontend&visibility=visible$/,
  );
  await expect(page.getByText("Frontend Studio")).toBeVisible();
  await expect(page.getByText("Parent")).toBeVisible();
  await expect(page.getByText("Security Response")).toHaveCount(0);

  const teamHref = await page
    .getByRole("link", { name: /Open Frontend Studio/ })
    .getAttribute("href");
  expect(teamHref).toBeTruthy();
  await page.goto(teamHref ?? "");
  await expect(
    page.getByRole("heading", { name: "Frontend Studio" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expect(page.getByText("Direct and inherited access")).toBeVisible();
  await expect(page.getByText("Hierarchy and mention delivery")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-detail-overview"),
  });

  await page.setViewportSize({ width: 390, height: 850 });
  await page.goto(`${seeded.hrefs.organizationProfile}/teams`);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-mobile"),
  });

  await signIn(page, seeded, "collaborator");
  await page.goto(`${seeded.hrefs.organizationProfile}/teams`);
  await expect(page.getByText("Platform Maintainers")).toBeVisible();
  await expect(page.getByText("Frontend Studio")).toBeVisible();
  await expect(page.getByText("Security Response")).toHaveCount(0);
  await expect(page.getByRole("link", { name: /Secret/ })).toHaveCount(0);

  await signIn(page, seeded);
  await page.goto(seeded.hrefs.organizationEmptyTeams);
  await expect(page.getByText("Organize people by team")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await expect(page.getByText("Flexible repository access")).toBeVisible();
  await expect(page.getByText("Request-to-join teams")).toBeVisible();
  await expect(page.getByText("Team mentions")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New team" }).first(),
  ).toHaveAttribute("href", /\/orgs\/org-empty-teams-[^/]+\/teams\/new$/);
  await expect(page.getByRole("link", { name: "Learn more" })).toHaveAttribute(
    "href",
    "/docs/api#organization-teams",
  );
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-empty-state"),
  });
});

test("organization team creation validates nesting and redirects to the created team", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  test.setTimeout(90_000);
  const seeded = await seed({ scenes: ["orgProfile"] });
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.organizationProfile}/teams/new`);
  await expect(
    page.getByRole("heading", { name: "Create team" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-create-form"),
  });
  await expect(page.getByLabel("Parent team")).toContainText("Platform");
  const parentValue =
    (await page
      .getByLabel("Parent team")
      .locator("option")
      .nth(1)
      .getAttribute("value")) ?? "";
  await page.getByLabel("Team name").fill("Release Infrastructure");
  await page.getByLabel("Description").fill("Owns release trains.");
  await page.getByLabel("Parent team").selectOption(parentValue);
  await page.getByLabel("Disabled").check();
  await page.getByRole("button", { name: "Create team" }).click();
  await expect(page).toHaveURL(
    /\/orgs\/org-profile-[^/]+\/teams\/release-infrastructure$/,
  );
  await expect(
    page.getByRole("heading", { name: "Release Infrastructure" }),
  ).toBeVisible();
  await expect(page.getByText("Fanout suppressed")).toBeVisible();

  await page.goto(`${seeded.hrefs.organizationProfile}/teams/new`);
  const nestedParentValue =
    (await page
      .getByLabel("Parent team")
      .locator("option")
      .nth(1)
      .getAttribute("value")) ?? "";
  await page.getByLabel("Team name").fill("Private Child");
  await page.getByLabel("Secret").check();
  await page.getByLabel("Parent team").selectOption(nestedParentValue);
  await expect(
    page.getByRole("button", { name: "Create team" }),
  ).toBeDisabled();

  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-004-final-validation-error"),
  });
});
