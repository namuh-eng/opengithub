import { execFileSync } from "node:child_process";
import { expect, type Locator, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededNavigation = {
  cookieName: string;
  cookieValue: string;
};

function seedNavigation(): SeededNavigation {
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
  return JSON.parse(output) as SeededNavigation;
}

async function signIn(page: Page, seeded: SeededNavigation) {
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

async function openFirstProjectWorkspace(page: Page) {
  await page.goto("/orgs/namuh/projects");
  await expect(page.getByRole("heading", { name: /Projects/i })).toBeVisible();
  const workspaceLink = page
    .locator('a[href*="/projects/"][href*="/views/"]')
    .first();
  await expect(workspaceLink).toBeVisible();
  await workspaceLink.click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

async function expectNoPageOverflow(page: Page) {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
}

async function projectItemLinks(page: Page): Promise<Locator[]> {
  return page.locator('a[href*="/projects/"][href*="/items/"]').all();
}

async function openDraftItemPanel(page: Page): Promise<Locator> {
  const links = await projectItemLinks(page);
  expect(links.length).toBeGreaterThan(0);

  for (const link of links.slice(0, 8)) {
    await link.click();
    const panel = page.getByRole("complementary", {
      name: "Project item detail",
    });
    await expect(panel).toBeVisible();
    if (await panel.getByText("Project-only draft").isVisible()) {
      return panel;
    }
    await panel.getByRole("link", { name: "Close" }).click();
  }

  throw new Error("Seeded Projects workspace did not include a draft item");
}

async function openAnyItemPanel(page: Page): Promise<Locator> {
  const links = await projectItemLinks(page);
  expect(links.length).toBeGreaterThan(0);
  await links[0].click();
  const panel = page.getByRole("complementary", {
    name: "Project item detail",
  });
  await expect(panel).toBeVisible();
  return panel;
}

test.skip(
  !databaseUrl,
  "Projects item side panel E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects item side panel supports final item lifecycle smoke", async ({
  page,
}) => {
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openFirstProjectWorkspace(page);

  await expect(page.getByRole("table")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoPageOverflow(page);

  const panel = await openDraftItemPanel(page);
  await expect(panel.getByText("Project-only draft")).toBeVisible();
  await expect(
    panel.getByRole("form", { name: "Edit draft project item" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-item-panel.jpg",
  });

  const suffix = Date.now();
  await panel.getByLabel("Title").fill(`QA draft ${suffix}`);
  await panel
    .getByLabel("Body")
    .fill("Updated by the final Projects item side-panel smoke.");
  await panel
    .getByRole("form", { name: "Edit draft project item" })
    .getByRole("button", { name: "Save draft" })
    .click();
  await expect(panel.getByText("Draft saved")).toBeVisible();
  await panel
    .getByPlaceholder("Add a project-only comment")
    .fill("Project-only comment from final smoke.");
  await panel
    .getByRole("form", { name: "Add project item comment" })
    .getByRole("button", { name: "Add comment" })
    .click();
  await expect(panel.getByText("Comment added")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-draft-editor.jpg",
  });

  await panel.getByRole("button", { name: "Convert to issue" }).click();
  await expect(
    panel.getByRole("form", { name: "Convert draft to issue" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-convert-dialog.jpg",
  });
  await panel.getByRole("button", { name: "Convert draft" }).click();
  await expect(panel.getByText("Draft converted to issue")).toBeVisible();
  await expect(panel.getByText("Project-only draft")).not.toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-linked-issue-sync.jpg",
  });

  await panel.getByRole("button", { name: "Archive" }).click();
  await expect(panel.getByText("Item archived")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-archive-confirmation.jpg",
  });

  await panel.getByRole("link", { name: "View archived items" }).click();
  await expect(
    page.getByRole("heading", { name: /Project archive/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Restore" }).first(),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-archived-list.jpg",
  });
  await page.getByRole("button", { name: "Restore" }).first().click();
  await expect(page.getByText("Item restored")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-restore-confirmation.jpg",
  });

  await page.getByRole("link", { name: /Back to project/i }).click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  const restoredPanel = await openAnyItemPanel(page);
  await expect(
    restoredPanel.getByRole("button", { name: "Remove" }),
  ).toBeVisible();
  await restoredPanel.getByRole("button", { name: "Remove" }).click();
  await expect(page).toHaveURL(/\/projects\/\d+\/views\/\d+/);
  await expect(page.getByText("Item removed from project.")).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  const mobilePanel = await openAnyItemPanel(page);
  await expect(mobilePanel).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoPageOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-005-final-mobile.jpg",
  });
});
