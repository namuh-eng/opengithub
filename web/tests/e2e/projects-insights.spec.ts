import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

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

async function openFirstProjectInsights(page: Page) {
  await page.goto("/orgs/namuh/projects");
  await expect(page.getByRole("heading", { name: /Projects/i })).toBeVisible();
  const insightsLink = page
    .locator('a[href*="/projects/"][href$="/insights"]')
    .first();
  await expect(insightsLink).toBeVisible();
  await insightsLink.click();
  await expect(
    page.getByRole("heading", { name: /Project insights|Editorial/i }),
  ).toBeVisible();
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

async function expectNoHorizontalOverflow(page: Page) {
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
}

test.skip(
  !databaseUrl,
  "Projects Insights E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("Projects Insights supports final chart exploration and custom chart smoke", async ({
  page,
}) => {
  const seeded = seedNavigation();
  await signIn(page, seeded);
  await openFirstProjectInsights(page);

  await expect(
    page.getByRole("link", { name: "Return to project view" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Insights" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("img", { name: /Burn up chart/i })).toBeVisible();
  await expect(page.getByText(/matching items/)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-008-final-burn-up.jpg",
  });

  await page.getByRole("link", { name: "2 weeks" }).click();
  await expect(page).toHaveURL(/range=2w/);
  await page.getByPlaceholder("is:open label:bug assignee:@me").fill("is:open");
  await page.getByRole("button", { name: "Apply filter" }).click();
  await expect(page).toHaveURL(/filter=is%3Aopen/);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-008-final-filter-range.jpg",
  });

  await page.getByText("Custom range").click();
  await page.getByLabel("Start date").fill("2026-04-01");
  await page.getByLabel("End date").fill("2026-05-06");
  await page.getByRole("button", { name: "Apply dates" }).click();
  await expect(page).toHaveURL(/range=custom/);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-008-final-custom-range.jpg",
  });

  const dataTableLink = page.getByRole("link", { name: "View as data table" });
  await expect(dataTableLink).toHaveAttribute("href", /table=true/);
  const dataTableHref = await dataTableLink.getAttribute("href");
  expect(dataTableHref).toBeTruthy();
  await page.goto(dataTableHref as string);
  await expect(page).toHaveURL(/table=true/);
  await expect(
    page.getByRole("table", { name: /chart data table/i }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-008-final-data-table.jpg",
  });

  const newChart = page.getByText("New");
  await expect(newChart).toBeVisible();
  await newChart.click();
  const createButton = page.getByRole("button", { name: "Create chart" });
  if (await createButton.isVisible()) {
    await page.getByLabel("Title").first().fill(`QA chart ${Date.now()}`);
    await page.getByLabel("Chart type").first().selectOption("line");
    await page.getByLabel("Visibility").first().selectOption("project");
    await page.screenshot({
      fullPage: true,
      path: "../ralph/screenshots/build/projects-008-final-custom-chart-form.jpg",
    });
    await createButton.click();
    await expect(page.getByRole("status")).toHaveText("Chart created.");
    await page.getByText("Edit", { exact: true }).click();
    await expect(
      page.getByRole("button", { name: "Save chart" }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Save chart" }).click();
    await expect(page.getByRole("status")).toHaveText("Chart saved.");
    await page.getByRole("button", { name: "Share" }).click();
    await expect(page.getByText(/Share link/)).toBeVisible();
    await page.screenshot({
      fullPage: true,
      path: "../ralph/screenshots/build/projects-008-final-shared-chart.jpg",
    });
    await page.getByRole("button", { name: "Delete" }).click();
    await expect(page.getByText("Chart deleted.")).toBeVisible();
  } else {
    await expect(
      page.getByText(/Your project role cannot create charts/),
    ).toBeVisible();
    await page.screenshot({
      fullPage: true,
      path: "../ralph/screenshots/build/projects-008-final-shared-read-only.jpg",
    });
  }

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByText("Project insights")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/projects-008-final-mobile.jpg",
  });
});
