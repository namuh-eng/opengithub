import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

function seedSession(marker: string): SeededSession {
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
        SEARCH_E2E_MARKER: marker,
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededSession;
}

async function signIn(page: Page, seeded: SeededSession) {
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

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test.skip(!databaseUrl, "search E2E needs TEST_DATABASE_URL or DATABASE_URL");

test("repository and people search render indexed results", async ({
  page,
}) => {
  const marker = `phase2-${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(`/search?q=${marker}&type=repositories`);
  await expect(
    page.getByRole("heading", { name: "Search opengithub" }),
  ).toBeVisible();
  await expect(page.getByText(/1 repositories results/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: new RegExp(marker) }),
  ).toHaveAttribute("href", /\/dash-.+\/search-.+/);

  await page.getByRole("link", { name: "Users" }).click();
  await expect(page).toHaveURL(new RegExp(`/search\\?q=${marker}&type=users$`));
  await expect(page.getByText(/1 users results/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Dashboard Tester/ }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Organizations" }).click();
  await expect(page).toHaveURL(
    new RegExp(`/search\\?q=${marker}&type=organizations$`),
  );
  await expect(page.getByText(/1 organizations results/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Search Organization/ }),
  ).toHaveAttribute("href", /\/orgs\/search-org-/);

  await page.goto(`/search?q=no-result-${marker}&type=repositories`);
  await expect(page.getByText(/Nothing matched/)).toBeVisible();
  await expect(page.getByText("owner:", { exact: true })).toBeVisible();
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-001-phase2-results.jpg",
  });
});

test("code and commit search link to repository files and commits", async ({
  page,
}) => {
  const marker = `phase3-${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(`/search?q=${marker}&type=code`);
  await expect(
    page.getByRole("heading", { name: "Search indexed code" }),
  ).toBeVisible();
  await expect(page.getByText("Result types")).toBeVisible();
  await expect(page.getByText("Languages")).toBeVisible();
  await expect(page.getByText("Paths")).toBeVisible();
  await expect(page.getByRole("link", { name: "Save" })).toHaveAttribute(
    "href",
    `/search?q=${marker}&type=code&saved=1`,
  );
  await expect(page.getByText(/1 code results/)).toBeVisible();
  const codeResult = page.getByRole("link", {
    name: /src\/search_phase_three\.rs/,
  });
  await expect(codeResult).toHaveAttribute(
    "href",
    /\/blob\/main\/src\/search_phase_three\.rs#L1/,
  );
  await expect(page.getByText(new RegExp(`pub fn ${marker}`))).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-003-phase2-code-results-shell.jpg",
  });
  await codeResult.click();
  await expect(page).toHaveURL(/\/blob\/main\/src\/search_phase_three\.rs/);
  await expect(
    page
      .locator("span")
      .filter({ hasText: new RegExp(`pub fn ${marker}`) })
      .first(),
  ).toBeVisible();

  await page.goto(`/search?q=${marker}&type=commits`);
  await expect(page.getByText(/1 commits results/)).toBeVisible();
  const commitResult = page.getByRole("link", {
    name: new RegExp(`Add ${marker} code search fixture`),
  });
  await expect(commitResult).toHaveAttribute("href", /\/commit\//);
  await expect(page.getByText(/Commit result for/)).toBeVisible();

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-001-phase3-code-results.jpg",
  });
});

test("issue, pull request, and discussions search tabs stay navigable", async ({
  page,
}) => {
  const marker = `phase4-${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto(`/search?q=${marker}&type=issues`);
  await expect(page.getByText(/1 issues results/)).toBeVisible();
  const issueResult = page.getByRole("link", {
    name: new RegExp(`Investigate ${marker} issue search`),
  });
  await expect(issueResult).toHaveAttribute("href", /\/issues\/1$/);
  await expect(issueResult.getByText("#1")).toBeVisible();
  await expect(issueResult.getByText("open", { exact: true })).toBeVisible();
  await expect(issueResult.getByText("bug")).toBeVisible();

  await page
    .getByRole("navigation", { name: "Search result types" })
    .getByRole("link", { name: "Pull requests" })
    .click();
  await expect(page).toHaveURL(
    new RegExp(`/search\\?q=${marker}&type=pull_requests$`),
  );
  await expect(page.getByText(/1 pull requests results/)).toBeVisible();
  const pullResult = page.getByRole("link", {
    name: new RegExp(`Review ${marker} pull search`),
  });
  await expect(pullResult).toHaveAttribute("href", /\/pull\/2$/);
  await expect(pullResult.getByText(`feature/${marker} -> main`)).toBeVisible();

  await page
    .getByRole("navigation", { name: "Search result types" })
    .getByRole("link", { name: "Discussions" })
    .click();
  await expect(page).toHaveURL(
    new RegExp(`/search\\?q=${marker}&type=discussions$`),
  );
  await expect(
    page.getByText("Discussion search is ready for indexing."),
  ).toBeVisible();
  await expectNoDeadControls(page);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-001-phase4-collaboration-results.jpg",
  });
});

test("final search sweep covers header submit, all tabs, mobile layout, and empty states", async ({
  page,
}) => {
  const marker = `phase5-${Date.now()}`;
  const seeded = seedSession(marker);
  await signIn(page, seeded);

  await page.goto("/dashboard");
  await page.getByRole("searchbox", { name: "Search or jump to" }).focus();
  const searchDialog = page.getByRole("dialog", { name: "Search" });
  await expect(searchDialog).toBeVisible();
  await expect(searchDialog.getByRole("listbox")).toBeVisible();
  await searchDialog
    .getByRole("combobox", { name: "Search opengithub" })
    .fill(marker);
  await expect(
    searchDialog.getByRole("option", { name: /Repositories/ }).first(),
  ).toHaveAttribute("href", /\/search\?q=/);
  await searchDialog.getByRole("link", { exact: true, name: "Search" }).click();
  await expect(page).toHaveURL(
    new RegExp(`/search\\?q=${marker}&type=repositories$`),
  );

  const expectedByType = [
    {
      label: "Repositories",
      type: "repositories",
      heading: "Search opengithub",
      text: /repositories results/,
    },
    {
      label: "Code",
      type: "code",
      heading: "Search indexed code",
      text: /code results/,
    },
    {
      label: "Issues",
      type: "issues",
      heading: "Issues search",
      text: /issues results/,
    },
    {
      label: "Pull requests",
      type: "pull_requests",
      heading: "Pull requests search",
      text: /pull requests results/,
    },
    {
      label: "Commits",
      type: "commits",
      heading: "Search opengithub",
      text: /commits results/,
    },
    {
      label: "Users",
      type: "users",
      heading: "Search opengithub",
      text: /users results/,
    },
    {
      label: "Organizations",
      type: "organizations",
      heading: "Search opengithub",
      text: /organizations results/,
    },
  ] as const;

  for (const tab of expectedByType) {
    await page.goto(`/search?q=${marker}&type=${tab.type}`);
    await expect(
      page.getByRole("heading", {
        name: tab.heading,
      }),
    ).toBeVisible();
    await expect(page.getByText(tab.text)).toBeVisible();
    const tabLink =
      tab.type === "code"
        ? page
            .getByRole("navigation", { name: "Search result types" })
            .getByRole("link", { name: /Code/ })
        : page
            .getByRole("navigation", { name: "Search result types" })
            .getByRole("link", { name: tab.label });
    await expect(tabLink).toHaveAttribute(
      "href",
      `/search?q=${marker}&type=${tab.type}`,
    );
    await expectNoDeadControls(page);
    const horizontalOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth > window.innerWidth,
    );
    expect(horizontalOverflow).toBe(false);
  }

  await page.goto(`/search?q=${marker}&type=discussions`);
  await expect(
    page.getByText("Discussion search is ready for indexing."),
  ).toBeVisible();
  await expectNoDeadControls(page);

  await page.goto(`/search?q=no-result-${marker}&type=code`);
  await expect(
    page.getByText(/Nothing in indexed files matched/),
  ).toBeVisible();
  await expect(page.getByText("language:", { exact: true })).toBeVisible();

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-001-phase5-final-desktop.jpg",
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/search?q=${marker}&type=repositories`);
  await expect(
    page.getByRole("heading", { name: "Search opengithub" }),
  ).toBeVisible();
  const mobileOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(mobileOverflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/search-001-phase5-final-mobile.jpg",
  });
});
