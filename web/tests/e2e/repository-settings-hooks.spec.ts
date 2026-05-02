import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;
const apiUrl = process.env.API_URL ?? "http://localhost:3016";

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

test("admin can view webhook list, detail, and delivery panels", async ({
  page,
  request,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);
  const receiverUrl = `https://receiver.opengithub.local/hooks/${Date.now()}`;
  const apiPath = `${apiUrl}/api/repos${seeded.firstRepositoryHref}/settings/hooks`;

  const createResponse = await request.post(apiPath, {
    data: {
      active: true,
      contentType: "json",
      eventSelection: "selected",
      events: ["push", "issues"],
      payloadUrl: receiverUrl,
      secret: "playwright-secret-value",
      sslVerify: true,
    },
    headers: {
      cookie: `${seeded.cookieName}=${seeded.cookieValue}`,
    },
  });
  expect(createResponse.status()).toBe(201);
  const created = (await createResponse.json()) as {
    delivery: { id: string };
    settings: { hooks: Array<{ id: string }> };
  };
  const hookId = created.settings.hooks[0].id;
  const deliveryId = created.delivery.id;

  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Webhooks" }),
  ).toBeVisible();
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
    path: "../ralph/screenshots/build/settings-004-phase2-hooks-list.jpg",
  });

  await page.goto(
    `${seeded.firstRepositoryHref}/settings/hooks/${hookId}?delivery=${deliveryId}`,
  );
  await expect(page.getByRole("heading", { name: receiverUrl })).toBeVisible();
  await expect(page.getByText("Recent deliveries")).toBeVisible();
  await expect(page.getByText("Request", { exact: true })).toBeVisible();
  await expect(page.getByText("Response", { exact: true })).toBeVisible();
  await expect(page.getByText(/Keep it logically awesome/)).toBeVisible();
  await expect(page.getByRole("link", { name: "Redeliver" })).toHaveAttribute(
    "href",
    `${seeded.firstRepositoryHref}/settings/hooks/${hookId}?delivery=${deliveryId}&redeliver=confirm`,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-phase2-hooks-detail.jpg",
  });

  await page.setViewportSize({ width: 390, height: 860 });
  await page.goto(`${seeded.firstRepositoryHref}/settings/hooks`);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/settings-004-phase2-hooks-mobile.jpg",
  });
});
