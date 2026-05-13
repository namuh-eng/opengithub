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
  "Projects board and roadmap E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.setTimeout(120_000);

async function openViewMenu(page: import("@playwright/test").Page) {
  await page.getByRole("button", { name: "View menu" }).click();
  await expect(page.getByRole("region", { name: "View menu" })).toBeVisible();
}

test("Projects board and roadmap layouts support final signed-in smoke", async ({
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
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

  await openViewMenu(page);
  await expect(page.getByRole("button", { name: /Table\s*t/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /Board\s*b/i })).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Roadmap\s*r/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-view-menu"),
  });

  await page.getByRole("button", { name: /Board\s*b/i }).click();
  await expect(page.getByRole("heading", { name: "Board" })).toBeVisible();
  await expect(page.getByText(/cards/i).first()).toBeVisible();
  await expect(
    page.getByText(/Board moves use the same project item field/),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-board-default"),
  });

  const moveSelect = page.getByLabel(/Move .* to column/).first();
  if (await moveSelect.isVisible()) {
    const options = await moveSelect.locator("option").all();
    if (options.length > 1) {
      const targetColumn = (await options[1].textContent())?.trim() ?? "";
      await moveSelect.selectOption({ index: 1 });
      await expect(page.getByRole("heading", { name: "Board" })).toBeVisible();
      await expect(
        page.getByRole("region", { name: `${targetColumn} board column` }),
      ).toContainText(/2 cards/);
    }
  }
  await page
    .getByRole("button", { name: /Show empty columns|Hide empty columns/ })
    .click();
  await page
    .getByRole("button", { name: /Add item/ })
    .first()
    .click();
  await expect(
    page.getByRole("textbox", { name: /Issue or pull request URL/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-board-move-add"),
  });
  await page.keyboard.press("Escape");

  await openViewMenu(page);
  await page.getByRole("button", { name: /Roadmap\s*r/i }).click();
  await expect(page.getByRole("heading", { name: "Roadmap" })).toBeVisible();
  await expect(
    page.getByRole("form", { name: "Roadmap settings" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /month/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /quarter/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /year/i })).toBeVisible();
  await page.getByRole("button", { name: /quarter/i }).click();
  await page.getByRole("button", { name: /Save roadmap/i }).click();
  await expect(page.getByRole("heading", { name: "Roadmap" })).toBeVisible();
  await expect(page.getByRole("button", { name: /quarter/i })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-roadmap-quarter"),
  });

  await page.getByRole("button", { name: /year/i }).click();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-roadmap-year"),
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "projects-003-final-mobile"),
  });
});
