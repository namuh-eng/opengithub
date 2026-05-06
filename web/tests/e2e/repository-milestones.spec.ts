import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

type CreatedMilestone = {
  id?: string;
  milestone?: { id: string; title: string };
  title?: string;
};

type CreatedIssue = {
  number: number;
};

type CreatedPullRequest = {
  number?: number;
  pull_request?: { number: number };
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

async function expectNoHorizontalOverflow(page: Page) {
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth + 1,
  );
  expect(horizontalOverflow).toBe(false);
}

function milestoneId(body: CreatedMilestone) {
  const id = body.id ?? body.milestone?.id;
  expect(id).toBeTruthy();
  return id as string;
}

test.skip(
  !databaseUrl,
  "Repository milestones E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in repository milestones manage lifecycle, assignment, reorder, and mobile layout", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const unique = Date.now().toString(36);
  const repositoryName = `milestones sweep ${unique}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page
    .getByLabel(/Description/)
    .fill("Repository for milestone management E2E coverage");
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));

  const [, ownerLogin, repoName] = new URL(page.url()).pathname.split("/");
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const milestoneResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/milestones`,
    {
      headers: { cookie },
      data: {
        title: `Launch readiness ${unique}`,
        description: "Track milestone E2E blockers.",
        dueOn: "2026-05-20T00:00:00Z",
      },
    },
  );
  expect(milestoneResponse.status()).toBe(201);
  const milestone = milestoneId(
    (await milestoneResponse.json()) as CreatedMilestone,
  );

  const issueOneResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: `Stabilize importer ${unique}`,
        body: "First open milestone item.",
        milestoneId: milestone,
      },
    },
  );
  expect(issueOneResponse.status()).toBe(201);
  const issueOne = (await issueOneResponse.json()) as CreatedIssue;

  const issueTwoResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: `Polish release notes ${unique}`,
        body: "Second open milestone item for reorder.",
        milestoneId: milestone,
      },
    },
  );
  expect(issueTwoResponse.status()).toBe(201);
  const issueTwo = (await issueTwoResponse.json()) as CreatedIssue;

  const pullResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/pulls`,
    {
      headers: { cookie },
      data: {
        title: `Milestone pull ${unique}`,
        body: "Pull request milestone assignment target.",
        headRef: "feature/milestone-sidebar",
        baseRef: "main",
        isDraft: false,
        milestoneId: milestone,
      },
    },
  );
  expect(pullResponse.status()).toBe(201);
  const pull = (await pullResponse.json()) as CreatedPullRequest;
  const pullNumber = pull.number ?? pull.pull_request?.number;
  expect(pullNumber).toBeTruthy();

  await page.goto(`/${ownerLogin}/${repoName}/milestones`);
  await expect(page.getByRole("heading", { name: "Milestones" })).toBeVisible();
  await expect(page.getByText(`Launch readiness ${unique}`)).toBeVisible();
  await page.getByText("Sort").click();
  await expect(
    page.getByRole("menuitemradio", { name: /Most issues/ }),
  ).toHaveAttribute("href", /sort=issues-desc/);
  await page.getByRole("button", { name: "New milestone" }).click();
  await page.getByLabel("Milestone title").fill(`Docs cutover ${unique}`);
  await page
    .getByLabel("Milestone description")
    .fill("Created through the final milestone E2E sweep.");
  await page.getByRole("button", { name: "Save milestone" }).click();
  await expect(page.getByText(`Docs cutover ${unique}`)).toBeVisible();
  await page.getByRole("button", { name: "Edit" }).first().click();
  await page
    .getByLabel("Milestone description")
    .fill("Edited through the final milestone E2E sweep.");
  await page.getByRole("button", { name: "Save milestone" }).click();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/milestones-001-final-desktop.jpg",
  });

  await page.goto(`/${ownerLogin}/${repoName}/milestones/${milestone}`);
  await expect(
    page.getByRole("heading", { name: `Launch readiness ${unique}` }),
  ).toBeVisible();
  await page.getByRole("link", { name: /Closed 0/ }).click();
  await expect(
    page.getByRole("heading", { name: "No closed items" }),
  ).toBeVisible();
  await page.getByRole("link", { name: /Open/ }).click();
  await expect(page.getByText(`Stabilize importer ${unique}`)).toBeVisible();
  await page
    .getByRole("button", {
      name: new RegExp(`Move Polish release notes ${unique} up`),
    })
    .click();
  await expect(page.getByText(`Polish release notes ${unique}`)).toBeVisible();
  await page.reload();
  await expect(page.getByText(`Polish release notes ${unique}`)).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();
  await page.reload();
  await expect(page.getByText("closed")).toBeVisible();
  await page.getByRole("button", { name: "Reopen" }).click();
  await page.reload();
  await expect(page.getByText("open")).toBeVisible();

  await page.getByRole("link", { name: "New issue" }).click();
  await expect(page).toHaveURL(
    new RegExp(`/issues/new\\?milestone=${milestone}`),
  );
  await page.goto(`/${ownerLogin}/${repoName}/issues/${issueOne.number}`);
  await page
    .locator("section", {
      has: page.getByRole("heading", { name: "Milestone" }),
    })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Search milestones").fill("No milestone");
  await page.getByRole("button", { name: /Choose No milestone/ }).click();
  await expect(page.getByText("Issue metadata updated.")).toBeVisible();
  await expect(page.getByText("No milestone")).toBeVisible();

  await page.goto(`/${ownerLogin}/${repoName}/pull/${pullNumber}`);
  await page
    .locator("section", {
      has: page.getByRole("heading", { name: "Milestone" }),
    })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Search milestones").fill("No milestone");
  await page.getByRole("button", { name: /Choose No milestone/ }).click();
  await expect(page.getByText("Pull request metadata updated.")).toBeVisible();
  await expect(page.getByText("No milestone")).toBeVisible();

  await page.goto(`/${ownerLogin}/${repoName}/issues/${issueTwo.number}`);
  await expect(page.getByText(`Launch readiness ${unique}`)).toBeVisible();
  await page.goto(`/${ownerLogin}/${repoName}/milestones/${milestone}`);
  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Delete" }).click();
  await expect(page).toHaveURL(new RegExp(`/${repoName}/milestones$`));

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/${ownerLogin}/${repoName}/milestones`);
  await expect(page.getByRole("heading", { name: "Milestones" })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/milestones-001-final-mobile.jpg",
  });
});
