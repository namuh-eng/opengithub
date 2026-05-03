import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

async function signIn(page: Page, seeded: SeededDashboard) {
  await page.context().addCookies([
    {
      name: seeded.cookieName,
      value: seeded.cookieValue,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      sameSite: "Lax",
      secure: false,
    },
  ]);
}

test.skip(
  !databaseUrl,
  "notifications E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in user marks notifications read and toggles saved state", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/notifications");
  const rowLink = page.getByRole("link", {
    name: /Triage dashboard setup workflow/,
  });
  await expect(rowLink).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page
    .getByRole("button", { name: "Save Triage dashboard setup workflow" })
    .click();
  await expect(page.getByRole("status")).toHaveText("Notification saved.");
  await expect(
    page.getByRole("button", {
      name: "Unsave Triage dashboard setup workflow",
    }),
  ).toBeVisible();

  await page
    .getByRole("button", {
      name: "Mark Triage dashboard setup workflow as read",
    })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Notification marked read.",
  );
  await expect(
    page.getByRole("button", {
      name: "Mark Triage dashboard setup workflow as unread",
    }),
  ).toBeVisible();

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-phase1-triage.jpg",
  });

  await page
    .getByRole("button", {
      name: "Move Triage dashboard setup workflow to Done",
    })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Notification moved to Done.",
  );
  await expect(rowLink).toHaveCount(0);

  await page.goto("/notifications?folder=done");
  await expect(rowLink).toBeVisible();
  await expect(
    page.getByRole("button", {
      name: "Move Triage dashboard setup workflow to inbox",
    }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-phase2-done.jpg",
  });

  await page
    .getByRole("button", {
      name: "Move Triage dashboard setup workflow to inbox",
    })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Notification moved to Inbox.",
  );
  await expect(rowLink).toHaveCount(0);

  await page.goto("/notifications");
  await expect(rowLink).toBeVisible();

  await page.goto("/notifications?folder=saved");
  await expect(rowLink).toBeVisible();

  await page
    .getByRole("button", {
      name: "Unsubscribe from Triage dashboard setup workflow",
    })
    .click();
  await expect(page.getByRole("status")).toHaveText("Thread unsubscribed.");
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-phase3-unsubscribed.jpg",
  });

  await page.goto("/notifications");
  await expect(rowLink).toHaveCount(0);

  await page.goto("/notifications?folder=saved");
  await page
    .getByRole("button", {
      name: "Subscribe to Triage dashboard setup workflow",
    })
    .click();
  await expect(page.getByRole("status")).toHaveText("Thread subscribed.");
  await page.goto("/notifications");
  await expect(rowLink).toBeVisible();
});

test("signed-in user selects notifications and runs bulk triage", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/notifications");
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();
  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await expect(page.getByText("1 selected")).toBeVisible();

  await page.getByRole("button", { exact: true, name: "Save" }).click();
  await expect(page.getByRole("status")).toHaveText("1 notification saved.");
  await expect(page.getByText("0 selected")).toBeVisible();

  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await page.getByRole("button", { exact: true, name: "Mark read" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "1 notification marked read.",
  );

  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await page.getByRole("button", { exact: true, name: "Done" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "1 notification moved to Done.",
  );
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-phase4-bulk.jpg",
  });

  await page.goto("/notifications?folder=done");
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();
  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await page
    .getByRole("button", { exact: true, name: "Move to inbox" })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "1 notification moved to Inbox.",
  );

  await page.goto("/notifications");
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();
  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await page.getByRole("button", { exact: true, name: "Mark unread" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "1 notification marked unread.",
  );

  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();
  await page.getByRole("button", { exact: true, name: "Unsubscribe" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "1 notification unsubscribed.",
  );
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/notifications?folder=saved");
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(horizontalOverflow).toBeLessThanOrEqual(2);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-final-mobile-saved.jpg",
  });
});

test("signed-in user keeps failed bulk rows selected for rollback", async ({
  page,
}) => {
  const seeded = seedDashboard();
  await signIn(page, seeded);

  await page.goto("/notifications");
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();
  await page
    .getByRole("checkbox", { name: "Select all visible notifications" })
    .click();

  await page.route(
    "**/notifications/bulk/triage",
    async (route) => {
      const payload = route.request().postDataJSON() as {
        notificationIds: string[];
      };
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          action: "save",
          updated: [],
          failed: [
            {
              id: payload.notificationIds[0],
              code: "notification_not_found",
              message: "Notification was not found.",
            },
          ],
          unreadCount: 1,
          folderCounts: { inbox: 1, saved: 0, done: 0 },
        }),
      });
    },
    { times: 1 },
  );

  await page.getByRole("button", { exact: true, name: "Save" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "0 notifications saved. 1 failed and stayed selected.",
  );
  await expect(page.getByText("1 selected")).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Triage dashboard setup workflow/ }),
  ).toBeVisible();

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-002-final-bulk-rollback.jpg",
  });
});
