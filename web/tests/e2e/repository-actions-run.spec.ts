import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(
  skipWithoutTestDb(),
  "repository Actions run E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.describe.configure({ timeout: 90_000 });

test("signed-in workflow run detail renders jobs, annotations, and artifacts", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["actionsRunDetail", "empty"] });
  await signIn(page, seeded, "owner");

  await page.goto(seeded.hrefs.actionsRunDetail);
  await expect(
    page.getByRole("heading", { name: /Validate Editorial CI/ }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("navigation", { name: "Repository" })
      .getByRole("link", { name: "Actions" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Editorial CI" })).toBeVisible();
  await expect(page.getByText("Workflow Dispatch on")).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Workflow run jobs" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: /Attempt 2/ })).toBeVisible();
  await expect(page.getByText("Type error")).toBeVisible();
  await expect(
    page.getByText("Expected string, found number", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText("playwright-report")).toBeVisible();
  await expect(page.getByText("sha256:abc123")).toBeVisible();
  await expect(page.getByText("Installing dependencies")).toBeVisible({
    timeout: 15_000,
  });
  await page.getByRole("textbox", { name: "Search job log" }).fill("error");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(
    page.getByText("error: Expected string, found number"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Download log" }),
  ).toHaveAttribute("href", /\/actions\/jobs\/.*\/logs\/download/);
  await expect(
    page.getByRole("link", { exact: true, name: "Download" }),
  ).toHaveAttribute("href", /\/actions\/artifacts\/.*\/download/);
  await page.getByRole("link", { name: /unit \/ web/ }).focus();
  await expect(page.getByRole("link", { name: /unit \/ web/ })).toBeFocused();
  await page.getByRole("button", { name: "Re-run failed" }).focus();
  await expect(
    page.getByRole("button", { name: "Re-run failed" }),
  ).toBeFocused();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-003-phase5-final-desktop.jpg",
  });

  await page.getByRole("button", { name: "Re-run failed" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Re Run Failed Jobs queued.",
  );
  await expect(page.getByRole("button", { name: "Cancel run" })).toBeEnabled();
  await page.getByRole("button", { name: "Cancel run" }).click();
  await expect(page.getByRole("status")).toContainText("Cancel Run queued.");
  await expect(page.getByRole("button", { name: "Delete logs" })).toBeEnabled();
  await page.getByRole("button", { name: "Delete logs" }).click();
  await expect(page.getByText(/Delete stored logs for this run/)).toBeVisible();
  await page.getByRole("button", { name: "Confirm delete" }).click();
  await expect(page.getByRole("status")).toContainText("Delete Logs queued.");
  await expect(page.getByText("Logs deleted").first()).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-003-phase4-run-actions.jpg",
  });

  await page.getByRole("link", { name: /deploy preview/ }).click();
  await expect(
    page.getByRole("heading", { exact: true, name: "deploy preview" }),
  ).toBeVisible();
  await expect(page.getByText("Logs deleted")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-003-phase3-logs-artifacts.jpg",
  });

  await page.setViewportSize({ height: 844, width: 390 });
  await page.reload();
  await expect(
    page.getByRole("heading", { name: /Validate Editorial CI/ }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-003-phase5-final-mobile.jpg",
  });
});
