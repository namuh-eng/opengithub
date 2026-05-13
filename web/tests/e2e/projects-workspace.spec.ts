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
  "Projects workspace E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.setTimeout(120_000);

test("Projects workspace table supports saved views, edits, add row, and final screenshots", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["projectsWorkspace"] });
  await signIn(page, seeded, "owner");
  expect(seeded.hrefs.projectsWorkspace).toMatch(
    /\/orgs\/namuh\/projects\/1\/views\/1/,
  );
  await page.goto(seeded.hrefs.projectsWorkspace);
  await expect(
    page.getByRole("heading", { level: 1, name: /workspace/i }),
  ).toBeVisible();

  await expect(
    page.getByRole("navigation", { name: /Saved project views/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("searchbox", { name: /Filter items/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /View configuration/i }),
  ).toBeVisible();
  await expect(page.getByRole("table")).toBeVisible();
  await expect(page.getByText(/matching items/i)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-002-final-default-table"),
  });

  await page.getByRole("button", { name: /View configuration/i }).click();
  await expect(
    page.getByRole("group", { name: /Visible fields/i }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /Save view/i })).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-002-final-view-config"),
  });
  await page
    .getByRole("button", { name: /Cancel|Revert/i })
    .first()
    .click();

  const editableCell = page
    .getByRole("button", { name: /Edit .* field/i })
    .first();
  await expect(editableCell).toBeVisible();
  await editableCell.click();
  await expect(page.getByRole("button", { name: /Save field/i })).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-002-final-inline-editor"),
  });
  await page.getByRole("button", { name: /Cancel field/i }).click();

  await page.getByRole("button", { name: /Add linked item/i }).click();
  await expect(
    page.getByRole("textbox", { name: /Issue or pull request URL/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-002-final-add-row"),
  });
  await page.keyboard.press("Escape");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-002-final-mobile"),
  });
});
