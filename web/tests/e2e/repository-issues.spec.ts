import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

type CreatedIssue = {
  id: string;
  number: number;
  title: string;
};

type IssueLabelOption = {
  id: string;
  name: string;
};

type IssueListResponse = {
  filterOptions: {
    labels: IssueLabelOption[];
  };
};

type CurrentUser = {
  id: string;
  username: string | null;
  email: string;
};

type CreatedRepository = {
  owner_login: string;
  name: string;
};

function seedSession(): SeededSession {
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
        DASHBOARD_E2E_EMPTY: "1",
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

async function openLabelsMenu(page: Page) {
  await page.getByRole("button", { name: /Labels/ }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter labels" }),
  ).toBeFocused();
}

test.skip(
  !databaseUrl,
  "repository issues E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in repository Issues tab renders real issues and row navigation", async ({
  browser,
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues repo ${Date.now().toString(36)}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page
    .getByLabel(/Description/)
    .fill("Repository for issue list smoke testing");
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));

  const [, ownerLogin, repoName] = new URL(page.url()).pathname.split("/");
  const issueTitle = `Default issue list smoke ${Date.now().toString(36)}`;
  const createResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: {
        cookie: `${seeded.cookieName}=${seeded.cookieValue}`,
      },
      data: {
        title: issueTitle,
        body: "Created through the real Rust API for the repository issue list.",
      },
    },
  );
  expect(createResponse.status()).toBe(201);
  const issue = (await createResponse.json()) as CreatedIssue;

  await page.goto(`/${ownerLogin}/${repoName}/issues`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Issues" }),
  ).toBeVisible();
  await expect(page.getByLabel("issue-query")).toHaveValue(
    "is:issue state:open",
  );
  await expect(page.getByRole("link", { name: /Open/ })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("link", { name: issueTitle })).toHaveAttribute(
    "href",
    `/${ownerLogin}/${repoName}/issues/${issue.number}`,
  );
  await expect(page.getByText(`#${issue.number}`)).toBeVisible();
  await expect(
    page.getByText(new RegExp(`${ownerLogin}/${repoName}`)),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "New issue" })).toHaveAttribute(
    "href",
    `/${ownerLogin}/${repoName}/issues/new`,
  );
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-001-phase2-open-list.jpg",
  });

  await page.getByRole("link", { name: issueTitle }).click();
  await expect(page).toHaveURL(
    new RegExp(`/${repoName}/issues/${issue.number}$`),
  );
  await expect(
    page.getByRole("heading", { name: new RegExp(issueTitle) }),
  ).toBeVisible();
  await expect(
    page.getByText("Created through the real Rust API"),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Assignees" })).toBeVisible();
  await expect(page.getByText("No milestone")).toBeVisible();
  await page
    .getByRole("textbox", { name: "Comment body" })
    .fill("Browser **guardrail** comment from Phase 5.");
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByText("Browser")).toBeVisible();
  await expect(page.getByText("guardrail")).toBeVisible();
  await page.getByRole("tab", { name: "Write" }).click();
  await page.getByRole("button", { name: "Comment" }).click();
  await expect(page.getByText("Comment posted.")).toBeVisible();
  await expect(page.getByText("guardrail")).toBeVisible();
  await page.getByRole("button", { name: "Subscribe" }).click();
  await expect(page.getByText("Subscribed to notifications.")).toBeVisible();
  await expect(page.getByText("Subscribed: subscribed")).toBeVisible();
  await page.getByRole("button", { name: "Customize" }).click();
  await expect(
    page.getByRole("heading", { name: "Customize updates" }),
  ).toBeVisible();
  await page.getByRole("checkbox", { name: /Closed/ }).check();
  await page.getByRole("checkbox", { name: /Reopened/ }).check();
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("Subscribed to notifications.")).toBeVisible();
  await expect(page.getByText("Custom events: closed, reopened")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-004-phase3-issue-customize.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/notifications-004-final-issue-customize.jpg",
  });
  await page.getByRole("button", { name: "Thumbs up 0" }).click();
  await expect(
    page.getByRole("button", { name: "Thumbs up 1" }),
  ).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Close issue" }).first().click();
  await expect(page.getByText("Issue closed.")).toBeVisible();
  await expect(page.locator(".chip", { hasText: "Closed" })).toBeVisible();
  await page.getByRole("button", { name: "Reopen issue" }).first().click();
  await expect(page.getByText("Issue reopened.")).toBeVisible();
  await expect(page.locator(".chip", { hasText: "Open" })).toBeVisible();
  const labelsSection = page.locator("section", {
    has: page.getByRole("heading", { name: "Labels" }),
  });
  await labelsSection.getByRole("button", { name: "Edit" }).click();
  await page.getByRole("checkbox", { name: /bug/ }).check();
  await page.getByRole("button", { name: "Save labels" }).click();
  await expect(page.getByText("Issue metadata updated.")).toBeVisible();
  await expect(page.locator(".chip", { hasText: "bug" })).toBeVisible();
  const assigneesSection = page.locator("section", {
    has: page.getByRole("heading", { name: "Assignees" }),
  });
  await assigneesSection.getByRole("button", { name: "Edit" }).click();
  await page
    .getByRole("button", { name: new RegExp(`Assign ${ownerLogin}`) })
    .click();
  await expect(page.getByText("Issue metadata updated.")).toBeVisible();
  await expect(assigneesSection.getByText(ownerLogin)).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-004-phase3-actions.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-004-phase4-metadata.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-004-phase5-final-desktop.jpg",
  });

  const anonymousPage = await browser.newPage();
  await anonymousPage.goto(`/${ownerLogin}/${repoName}/issues/${issue.number}`);
  await expect(
    anonymousPage.getByRole("heading", { name: new RegExp(issueTitle) }),
  ).toBeVisible();
  await expect(
    anonymousPage
      .locator(
        `a[href="/login?next=%2F${ownerLogin}%2F${repoName}%2Fissues%2F${issue.number}"]`,
      )
      .first(),
  ).toBeVisible();
  await expect(
    anonymousPage.getByRole("link", { name: "Sign in to subscribe" }),
  ).toHaveAttribute(
    "href",
    `/login?next=%2F${ownerLogin}%2F${repoName}%2Fissues%2F${issue.number}`,
  );
  await expectNoDeadControls(anonymousPage);
  await anonymousPage.close();

  const mobilePage = await browser.newPage({
    viewport: { width: 390, height: 844 },
  });
  await signIn(mobilePage, seeded);
  await mobilePage.goto(`/${ownerLogin}/${repoName}/issues/${issue.number}`);
  await expect(
    mobilePage.getByRole("heading", { name: new RegExp(issueTitle) }),
  ).toBeVisible();
  await expectNoDeadControls(mobilePage);
  const overflow = await mobilePage.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await mobilePage.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-004-phase5-final-mobile.jpg",
  });
  await mobilePage.close();
});

test("signed-in repository Issues filters update URL, results, and empty states", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues filters ${Date.now().toString(36)}`;
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const currentUserResponse = await page.request.get(
    "http://localhost:3016/api/auth/current-user",
    { headers: { cookie } },
  );
  expect(currentUserResponse.status()).toBe(200);
  const currentUser = (await currentUserResponse.json()) as CurrentUser;
  const repositoryResponse = await page.request.post(
    "http://localhost:3016/api/repos",
    {
      headers: { cookie },
      data: {
        ownerType: "user",
        ownerId: currentUser.id,
        name: repositoryName,
        visibility: "public",
        initializeReadme: false,
      },
    },
  );
  expect(repositoryResponse.status()).toBe(201);
  const repository = (await repositoryResponse.json()) as CreatedRepository;
  const ownerLogin = repository.owner_login;
  const repoName = repository.name;
  const openTitle = `Filter target issue ${Date.now().toString(36)}`;
  const closedTitle = `Closed target issue ${Date.now().toString(36)}`;
  const openResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: openTitle,
        body: "Repository issue filter smoke body",
      },
    },
  );
  expect(openResponse.status()).toBe(201);
  const closedResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: closedTitle,
        body: "Closed state filter smoke body",
      },
    },
  );
  expect(closedResponse.status()).toBe(201);
  const closedIssue = (await closedResponse.json()) as CreatedIssue;
  const closeResponse = await page.request.patch(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues/${closedIssue.number}`,
    {
      headers: { cookie },
      data: { state: "closed" },
    },
  );
  expect(closeResponse.status()).toBe(200);

  await page.goto(`/${ownerLogin}/${repoName}/issues`);
  await page.getByLabel("issue-query").fill("target issue");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/q=target\+issue/);
  await expect(page.getByRole("link", { name: openTitle })).toBeVisible();
  await expect(page.getByRole("link", { name: closedTitle })).toHaveCount(0);

  await page.getByRole("link", { name: /Closed/ }).click();
  await expect(page).toHaveURL(/state=closed/);
  await expect(page.getByRole("link", { name: closedTitle })).toBeVisible();

  await page.getByLabel("issue-query").fill("no matching issue text");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page.getByText("No issues matched this query")).toBeVisible();
  await page.getByRole("link", { name: "Clear query" }).click();
  await expect(page).toHaveURL(/q=is%3Aissue\+state%3Aopen/);
  await expect(page.getByRole("link", { name: openTitle })).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-001-phase3-filtered-empty.jpg",
  });
});

test("signed-in repository Issues label menu filters, excludes, and finds unlabeled issues", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues labels ${Date.now().toString(36)}`;
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const currentUserResponse = await page.request.get(
    "http://localhost:3016/api/auth/current-user",
    { headers: { cookie } },
  );
  expect(currentUserResponse.status()).toBe(200);
  const currentUser = (await currentUserResponse.json()) as CurrentUser;
  const repositoryResponse = await page.request.post(
    "http://localhost:3016/api/repos",
    {
      headers: { cookie },
      data: {
        ownerType: "user",
        ownerId: currentUser.id,
        name: repositoryName,
        visibility: "public",
        initializeReadme: false,
      },
    },
  );
  expect(repositoryResponse.status()).toBe(201);
  const repository = (await repositoryResponse.json()) as CreatedRepository;
  const ownerLogin = repository.owner_login;
  const repoName = repository.name;
  const unlabeledTitle = `Unlabeled menu issue ${Date.now().toString(36)}`;
  const unlabeledResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: unlabeledTitle,
        body: "This issue intentionally has no labels.",
      },
    },
  );
  expect(unlabeledResponse.status()).toBe(201);

  const optionsResponse = await page.request.get(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    { headers: { cookie } },
  );
  expect(optionsResponse.status()).toBe(200);
  const optionsBody = (await optionsResponse.json()) as IssueListResponse;
  const bugLabel = optionsBody.filterOptions.labels.find(
    (label) => label.name === "bug",
  );
  if (!bugLabel) {
    throw new Error("bug label option should be present");
  }
  const bugTitle = `Bug menu issue ${Date.now().toString(36)}`;
  const bugResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: bugTitle,
        body: "This issue should be selected by the label menu.",
        labelIds: [bugLabel.id],
      },
    },
  );
  expect(bugResponse.status()).toBe(201);

  await page.goto(`/${ownerLogin}/${repoName}/issues`);
  const issuesPath = `/${ownerLogin}/${repoName}/issues`;

  await openLabelsMenu(page);
  await page.getByRole("combobox", { name: "Filter labels" }).fill("bug");
  await expect(page.getByRole("option", { name: /bug/ })).toBeVisible();
  await page.getByRole("option", { name: /bug/ }).click();
  await expect(page).toHaveURL(/labels=bug/);
  await expect(page.getByRole("link", { name: bugTitle })).toBeVisible();
  await expect(page.getByRole("link", { name: unlabeledTitle })).toHaveCount(0);

  await page.goto(issuesPath);
  await openLabelsMenu(page);
  await page.getByRole("combobox", { name: "Filter labels" }).fill("bug");
  await page.getByRole("option", { name: /bug/ }).click({ modifiers: ["Alt"] });
  await expect(page).toHaveURL(/excludedLabels=bug/);
  await expect(page.getByRole("link", { name: unlabeledTitle })).toBeVisible();
  await expect(page.getByRole("link", { name: bugTitle })).toHaveCount(0);

  await page.goto(issuesPath);
  await openLabelsMenu(page);
  await page.getByRole("option", { name: /No labels/ }).click();
  await expect(page).toHaveURL(/noLabels=true/);
  await expect(page.getByRole("link", { name: unlabeledTitle })).toBeVisible();
  await expect(page.getByRole("link", { name: bugTitle })).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-002-phase1-label-menu.jpg",
  });
});

test("signed-in repository Issues people and metadata menus update filters", async ({
  browser,
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues people ${Date.now().toString(36)}`;
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const currentUserResponse = await page.request.get(
    "http://localhost:3016/api/auth/current-user",
    { headers: { cookie } },
  );
  expect(currentUserResponse.status()).toBe(200);
  const currentUser = (await currentUserResponse.json()) as CurrentUser;
  const repositoryResponse = await page.request.post(
    "http://localhost:3016/api/repos",
    {
      headers: { cookie },
      data: {
        ownerType: "user",
        ownerId: currentUser.id,
        name: repositoryName,
        visibility: "public",
        initializeReadme: false,
      },
    },
  );
  expect(repositoryResponse.status()).toBe(201);
  const repository = (await repositoryResponse.json()) as CreatedRepository;
  const issueTitle = `People menu issue ${Date.now().toString(36)}`;
  const issueResponse = await page.request.post(
    `http://localhost:3016/api/repos/${repository.owner_login}/${repository.name}/issues`,
    {
      headers: { cookie },
      data: {
        title: issueTitle,
        body: "This issue should remain reachable through metadata filters.",
      },
    },
  );
  expect(issueResponse.status()).toBe(201);

  const issuesUrl = `/${repository.owner_login}/${repository.name}/issues`;
  await page.goto(issuesUrl);
  await page.getByRole("button", { name: /Author/ }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter authors" }),
  ).toBeFocused();
  const currentLogin = currentUser.username ?? currentUser.email;
  await page.getByRole("option", { name: new RegExp(currentLogin) }).click();
  await expect(page).toHaveURL(/author=/);
  await expect(page.getByRole("link", { name: issueTitle })).toBeVisible();

  await page.goto(issuesUrl);
  await page.getByRole("button", { name: /Assignees/ }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter assignees" }),
  ).toBeFocused();
  await page.getByRole("option", { name: /No assignee/ }).click();
  await expect(page).toHaveURL(/noAssignee=true/);
  await expect(page.getByRole("link", { name: issueTitle })).toBeVisible();

  await page.goto(issuesUrl);
  await page.getByRole("button", { name: /Milestones/ }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter milestones" }),
  ).toBeFocused();
  await page.getByRole("option", { name: /No milestone/ }).click();
  await expect(page).toHaveURL(/noMilestone=true/);
  await expect(page.getByRole("link", { name: issueTitle })).toBeVisible();

  await page.getByRole("button", { name: /Projects/ }).click();
  await expect(
    page.getByRole("option", { name: /No repository projects/ }),
  ).toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: /Types/ }).click();
  await expect(
    page.getByRole("option", { name: /No issue types/ }),
  ).toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: /Sort by/ }).click();
  await expect(page.getByRole("menu", { name: "Sort issues" })).toBeVisible();
  await expect(
    page.getByRole("menuitemradio", { name: /Recently updated/ }),
  ).toHaveAttribute("aria-checked", "true");
  await page.getByRole("menuitemradio", { name: /Most commented/ }).click();
  await expect(page).toHaveURL(/sort=comments-desc/);
  await expect(
    page.getByRole("button", { name: /Sort by: Most commented/ }),
  ).toBeVisible();

  await page.goto(
    `${issuesUrl}?q=${encodeURIComponent("is:issue state:merged")}`,
  );
  await expect(
    page.locator('div[role="alert"]').filter({ hasText: "Query warning" }),
  ).toContainText("state filter must be open or closed");
  await expect(page.getByLabel("issue-query")).toHaveValue(
    "is:issue state:merged",
  );
  await expect(page.getByRole("button", { name: /Labels/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /Sort by/ })).toBeVisible();
  await page.getByRole("link", { name: "Clear invalid query" }).click();
  await expect(page).toHaveURL(/q=is%3Aissue\+state%3Aopen/);
  await expect(page.getByRole("link", { name: issueTitle })).toBeVisible();
  await expectNoDeadControls(page);
  await page.getByRole("button", { name: /Labels/ }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter labels" }),
  ).toBeFocused();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-002-phase5-final-desktop.jpg",
  });

  const mobilePage = await browser.newPage({
    viewport: { width: 390, height: 844 },
  });
  await signIn(mobilePage, seeded);
  await mobilePage.goto(issuesUrl);
  await mobilePage.getByRole("button", { name: /Sort by/ }).click();
  await expect(
    mobilePage.getByRole("menu", { name: "Sort issues" }),
  ).toBeVisible();
  const overflow = await mobilePage.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await expectNoDeadControls(mobilePage);
  await mobilePage.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-002-phase5-final-mobile.jpg",
  });
  await mobilePage.close();
});

test("signed-in repository Issues contributor banner dismissal persists on reload", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues banner ${Date.now().toString(36)}`;
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const currentUserResponse = await page.request.get(
    "http://localhost:3016/api/auth/current-user",
    { headers: { cookie } },
  );
  expect(currentUserResponse.status()).toBe(200);
  const currentUser = (await currentUserResponse.json()) as CurrentUser;
  const repositoryResponse = await page.request.post(
    "http://localhost:3016/api/repos",
    {
      headers: { cookie },
      data: {
        ownerType: "user",
        ownerId: currentUser.id,
        name: repositoryName,
        visibility: "public",
        initializeReadme: false,
      },
    },
  );
  expect(repositoryResponse.status()).toBe(201);
  const repository = (await repositoryResponse.json()) as CreatedRepository;

  await page.goto(`/${repository.owner_login}/${repository.name}/issues`);
  await expect(
    page.getByRole("region", { name: "Contributor guidance" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Dismiss" }).click();
  await expect(
    page.getByRole("region", { name: "Contributor guidance" }),
  ).toHaveCount(0);

  await page.reload();
  await expect(
    page.getByRole("region", { name: "Contributor guidance" }),
  ).toHaveCount(0);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-001-phase4-banner-dismissed.jpg",
  });
});

test("public repository Issues are readable anonymously and fit mobile", async ({
  browser,
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `issues public ${Date.now().toString(36)}`;
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const currentUserResponse = await page.request.get(
    "http://localhost:3016/api/auth/current-user",
    { headers: { cookie } },
  );
  expect(currentUserResponse.status()).toBe(200);
  const currentUser = (await currentUserResponse.json()) as CurrentUser;
  const repositoryResponse = await page.request.post(
    "http://localhost:3016/api/repos",
    {
      headers: { cookie },
      data: {
        ownerType: "user",
        ownerId: currentUser.id,
        name: repositoryName,
        visibility: "public",
        initializeReadme: false,
      },
    },
  );
  expect(repositoryResponse.status()).toBe(201);
  const repository = (await repositoryResponse.json()) as CreatedRepository;
  const issueTitle = `Anonymous public issue ${Date.now().toString(36)}`;
  const issueResponse = await page.request.post(
    `http://localhost:3016/api/repos/${repository.owner_login}/${repository.name}/issues`,
    {
      headers: { cookie },
      data: {
        title: issueTitle,
        body: "Anonymous readers should see this public repository issue.",
      },
    },
  );
  expect(issueResponse.status()).toBe(201);

  const anonymousPage = await browser.newPage();
  await anonymousPage.goto(
    `/${repository.owner_login}/${repository.name}/issues`,
  );
  await expect(
    anonymousPage.getByRole("heading", { exact: true, name: "Issues" }),
  ).toBeVisible();
  await expect(
    anonymousPage.getByRole("link", { name: issueTitle }),
  ).toBeVisible();
  await expect(
    anonymousPage.getByRole("button", { name: "Dismiss" }),
  ).toBeVisible();
  await expectNoDeadControls(anonymousPage);
  await anonymousPage.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-001-phase5-public-anonymous.jpg",
  });
  await anonymousPage.close();

  const mobilePage = await browser.newPage({
    viewport: { width: 390, height: 844 },
  });
  await mobilePage.goto(`/${repository.owner_login}/${repository.name}/issues`);
  await expect(
    mobilePage.getByRole("link", { name: issueTitle }),
  ).toBeVisible();
  const overflow = await mobilePage.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(overflow).toBe(false);
  await expectNoDeadControls(mobilePage);
  await mobilePage.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/issues-001-phase5-mobile.jpg",
  });
  await mobilePage.close();
});
