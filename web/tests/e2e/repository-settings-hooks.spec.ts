import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref: string;
};

function seedDashboard(): SeededDashboard {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }

  const output = execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "-p",
      "opengithub-api",
      "--example",
      "dashboard_e2e_seed",
    ],
    {
      cwd: "..",
      env: {
        ...process.env,
        DASHBOARD_E2E_EMPTY: "0",
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard) {
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: seeded.cookieValue,
    },
  ]);
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test.skip(
  !databaseUrl,
  "repository webhook settings smoke needs TEST_DATABASE_URL or DATABASE_URL",
);

test("admin can create, edit, test, redeliver, and delete a webhook", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const receiverUrl = `https://receiver.example.com/hooks/${Date.now()}`;

  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks?new=webhook`);
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

  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks`);
  await expect(page.getByText("Configured endpoints")).toBeVisible();
  await expect(page.getByRole("link", { name: receiverUrl })).toHaveAttribute(
    "href",
    `${seeded.firstRepositoryHref}/settings/hooks/${hookId}`,
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

  await page.goto(
    `${seeded.firstRepositoryHref}/settings/hooks/${hookId}?delivery=${deliveryId}`,
  );
  await expect(page.getByRole("heading", { name: receiverUrl })).toBeVisible();
  await expect(page.getByText("Recent deliveries")).toBeVisible();
  await expect(page.getByText("Request", { exact: true })).toBeVisible();
  await expect(page.getByText("Response", { exact: true })).toBeVisible();
  await expect(page.getByText(/Keep it logically awesome/)).toBeVisible();

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

  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks`);
  const row = page.locator(".list-row", { hasText: receiverUrl });
  await row.getByRole("button", { name: "Delete" }).click();
  await expect(
    page.getByRole("button", { name: "Delete webhook" }),
  ).toBeDisabled();
  await page.getByLabel("Type payload URL to confirm").fill(receiverUrl);
  await page.getByRole("button", { name: "Delete webhook" }).click();
  await expect(page.getByText("Webhook deleted.")).toBeVisible();
  await expect(row).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 860 });
  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks`);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-phase3-hooks-mobile.jpg",
  });
});
