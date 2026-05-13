import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededNavigation = {
  cookieName: string;
  cookieValue: string;
  projectsWorkspaceHref: string;
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
        PROJECTS_WORKSPACE_E2E: "1",
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

async function openFirstProjectWorkspace(page: Page, workspaceHref: string) {
  expect(workspaceHref).toMatch(/\/orgs\/namuh\/projects\/\d+\/views\/\d+/);
  await expect
    .poll(
      async () => {
        await page.goto(workspaceHref);
        const heading = page.getByRole("heading", { level: 1 }).first();
        await heading.waitFor({ state: "visible", timeout: 10_000 });
        return (await heading.textContent())?.trim();
      },
      {
        message: "seeded Projects workspace route is ready",
        timeout: 60_000,
      },
    )
    .toBe("Editorial table workspace");
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

async function openViewMenu(page: Page) {
  await page.getByRole("button", { name: "View menu" }).click();
  await expect(page.getByRole("region", { name: "View menu" })).toBeVisible();
}

test.skip(
  !databaseUrl,
  "Projects board and roadmap E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects board and roadmap layouts support final signed-in smoke", async ({
  page,
}) => {
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openFirstProjectWorkspace(page, seeded.projectsWorkspaceHref);

  await openViewMenu(page);
  await expect(page.getByRole("button", { name: /Table\s*t/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /Board\s*b/i })).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Roadmap\s*r/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-003-final-view-menu.jpg",
  });

  await page.getByRole("button", { name: /Board\s*b/i }).click();
  await expect(page.getByRole("heading", { name: "Board" })).toBeVisible();
  await expect(page.getByText(/cards/i).first()).toBeVisible();
  await expect(
    page.getByText(/Board moves use the same project item field/),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoPageOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-003-final-board-default.jpg",
  });

  const moveSelect = page.getByLabel(/Move .* to column/).first();
  if (await moveSelect.isVisible()) {
    const options = await moveSelect.locator("option").all();
    if (options.length > 1) {
      const nextValue = await options[1].getAttribute("value");
      expect(nextValue).toBeTruthy();
      await moveSelect.selectOption({ index: 1 });
      await page.waitForLoadState("networkidle");
      await expect(
        page.getByLabel(/Move Wire the table shell to column/),
      ).toHaveValue(nextValue ?? "");
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
    path: "../ralph/screenshots/build/projects-003-final-board-move-add.jpg",
  });
  await page.keyboard.press("Escape");

  await openViewMenu(page);
  await page.getByRole("button", { name: /Roadmap\s*r/i }).click();
  await expect(page.getByRole("heading", { name: "Roadmap" })).toBeVisible();
  await expect(
    page.getByRole("form", { name: "Roadmap settings" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /Month/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /Quarter/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /Year/i })).toBeVisible();
  await page.getByRole("button", { name: /Quarter/i }).click();
  await page.getByRole("button", { name: /Save roadmap/i }).click();
  await page.waitForLoadState("networkidle");
  await expect(page.getByRole("button", { name: /Quarter/i })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-003-final-roadmap-quarter.jpg",
  });

  await page.getByRole("button", { name: /Year/i }).click();
  await expectNoDeadControls(page);
  await expectNoPageOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-003-final-roadmap-year.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoPageOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-003-final-mobile.jpg",
  });
});
