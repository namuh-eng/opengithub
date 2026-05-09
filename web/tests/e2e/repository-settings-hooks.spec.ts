import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(
  skipWithoutTestDb(),
  "repository webhook settings smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can create, edit, test, redeliver, and delete a webhook", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded, "owner");
  const receiverUrl = `https://receiver.example.com/hooks/${Date.now()}`;

  await page.goto(`${seeded.hrefs.firstRepository}/settings/hooks?new=webhook`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Webhooks" }),
  ).toBeVisible();
  await page.getByLabel("Payload URL").fill(receiverUrl);
  await page.getByLabel("Secret").fill("playwright-secret-value");
  await page.getByLabel("Let me select individual events").check();
  await expect(page.getByLabel(/Pushes/)).toBeVisible();
  await page.getByLabel(/Pushes/).check();
  await page.getByLabel(/Issues/).check();
  await page.getByRole("button", { name: "Add webhook" }).click();
  await expect(page).toHaveURL(/\/settings\/hooks\/[^/]+\?delivery=/);
  const hookUrl = page.url();
  const hookId = hookUrl.match(/\/settings\/hooks\/([^?]+)/)?.[1];
  const deliveryId = new URL(hookUrl).searchParams.get("delivery");
  expect(hookId).toBeTruthy();
  expect(deliveryId).toBeTruthy();

  await page.goto(`${seeded.hrefs.firstRepository}/settings/hooks`);
  await expect(page.getByText("Configured endpoints")).toBeVisible();
  await expect(page.getByRole("link", { name: receiverUrl })).toHaveAttribute(
    "href",
    `${seeded.hrefs.firstRepository}/settings/hooks/${hookId}`,
  );
  await expect(page.getByText(/Pushes|Issues/).first()).toBeVisible();
  await expect(
    page.getByText(/queued|delivered|failed/i).first(),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-phase3-hooks-mutations.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-final-hooks-list-admin.jpg",
  });

  await page.goto(
    `${seeded.hrefs.firstRepository}/settings/hooks/${hookId}?delivery=${deliveryId}`,
  );
  await expect(page.getByRole("heading", { name: receiverUrl })).toBeVisible();
  await expect(page.getByText("Recent deliveries")).toBeVisible();
  await expect(page.getByText("Request", { exact: true })).toBeVisible();
  await expect(page.getByText("Response", { exact: true })).toBeVisible();
  await expect(page.getByText(/Keep it logically awesome/)).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-final-hooks-detail-admin.jpg",
  });

  await page.getByRole("button", { name: "Edit" }).click();
  await page.getByLabel("Just push").check();
  await page.getByLabel("Active").uncheck();
  await page.getByRole("button", { name: "Edit webhook" }).click();
  await expect(page.getByText("Webhook settings saved.")).toBeVisible();
  await expect(page.getByText("Inactive")).toBeVisible();

  await page.getByRole("button", { name: "Test" }).click();
  await page.getByRole("button", { name: "Send ping" }).click();
  await expect(page.getByText("Webhook ping delivery queued.")).toBeVisible();

  await page.getByRole("button", { name: "Redeliver" }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Redeliver" })
    .click();
  await expect(page.getByText("Webhook delivery queued again.")).toBeVisible();
  await expectNoDeadControls(page);

  await page.goto(`${seeded.hrefs.firstRepository}/settings/hooks`);
  const row = page.locator(".list-row", { hasText: receiverUrl });
  await row.getByRole("button", { name: "Delete" }).click();
  await expect(
    page.getByRole("button", { name: "Delete webhook" }),
  ).toBeDisabled();
  await page.getByLabel("Type payload URL to confirm").fill(receiverUrl);
  await page.getByRole("button", { name: "Delete webhook" }).click();
  await expect(page.getByText("Webhook deleted.")).toBeVisible();
  await expect(row).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-final-hooks-empty-admin.jpg",
  });

  await page.setViewportSize({ width: 390, height: 860 });
  await page.goto(`${seeded.hrefs.firstRepository}/settings/hooks`);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-phase3-hooks-mobile.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-final-hooks-mobile.jpg",
  });

  await page.context().clearCookies();
  await signIn(page, seeded, "collaborator");
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(`${seeded.hrefs.firstRepository}/settings/hooks`);
  await expect(
    page.getByRole("heading", { name: "Webhook settings are restricted" }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-final-hooks-forbidden.jpg",
  });
});
