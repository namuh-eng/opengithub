import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
};

type CreatedWorkflow = {
  id: string;
};

type CreatedRun = {
  id: string;
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

async function createRepository(page: Page, name: string) {
  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(name);
  await page.getByLabel(/Description/).fill("Actions workflow smoke testing");
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${name.replaceAll(/\s+/g, "-")}$`));
  const [, ownerLogin, repoName] = new URL(page.url()).pathname.split("/");
  return { ownerLogin, repoName };
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
}

test.skip(
  !databaseUrl,
  "repository Actions E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in repository Actions tab renders workflows, runs, and empty templates", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `actions repo ${Date.now().toString(36)}`;
  const { ownerLogin, repoName } = await createRepository(page, repositoryName);
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;

  const workflowResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/actions/workflows`,
    {
      headers: { cookie },
      data: {
        name: "Editorial CI",
        path: ".github/workflows/editorial-ci.yml",
        triggerEvents: ["push", "pull_request"],
      },
    },
  );
  expect(workflowResponse.status()).toBe(201);
  const workflow = (await workflowResponse.json()) as CreatedWorkflow;
  const runResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/actions/workflows/${workflow.id}/runs`,
    {
      headers: { cookie },
      data: {
        headBranch: "main",
        headSha: "abcdef1234567890",
        event: "push",
      },
    },
  );
  expect(runResponse.status()).toBe(201);
  const run = (await runResponse.json()) as CreatedRun;
  const transitionResponse = await page.request.patch(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/actions/runs/${run.id}`,
    {
      headers: { cookie },
      data: {
        status: "completed",
        conclusion: "success",
      },
    },
  );
  expect(transitionResponse.status()).toBe(200);

  await page.goto(`/${ownerLogin}/${repoName}/actions`);
  await expect(
    page.getByRole("heading", { name: "All workflows" }),
  ).toBeVisible();
  const workflowNav = page.getByRole("navigation", {
    name: "Actions workflows",
  });
  await expect(
    workflowNav.getByRole("link", { name: /Editorial CI/ }),
  ).toHaveAttribute("href", new RegExp(`/actions\\?workflow=${workflow.id}`));
  await expect(
    page.getByRole("link", { exact: true, name: "Editorial CI" }),
  ).toBeVisible();
  await expect(page.getByLabel("Success run")).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "Editorial CI" }),
  ).toHaveAttribute("href", new RegExp(`/actions/runs/${run.id}`));
  await expect(page.getByText("push")).toBeVisible();
  await expect(page.getByText("main")).toBeVisible();
  await page.getByPlaceholder("Filter workflow runs").fill("Editorial");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/q=Editorial/);
  await expect(
    page.getByRole("link", { exact: true, name: "Editorial CI" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Status" }).click();
  await page.getByRole("menuitemradio", { name: /success/i }).click();
  await expect(page).toHaveURL(/status=success/);
  await page.getByRole("button", { name: "Branch" }).click();
  await page.getByRole("menuitemradio", { name: /main/i }).click();
  await expect(page).toHaveURL(/branch=main/);
  await workflowNav.getByRole("link", { name: /Editorial CI/ }).click();
  await expect(page).toHaveURL(new RegExp(`workflow=${workflow.id}`));
  const recentView = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/actions/recent-view`,
    {
      headers: { cookie },
      data: {
        branch: "main",
        q: "Editorial",
        status: "success",
        workflow: workflow.id,
      },
    },
  );
  expect(recentView.status()).toBe(200);
  await page.getByRole("link", { name: "Caches" }).click();
  await expect(
    page.getByRole("heading", { name: "Actions caches" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "All workflows" }),
  ).toHaveAttribute("href", `/${ownerLogin}/${repoName}/actions`);
  await page.goto(`/${ownerLogin}/${repoName}/actions`);
  await page.getByRole("link", { name: "Deployments" }).click();
  await expect(
    page.getByRole("heading", { name: "Actions deployments" }),
  ).toBeVisible();
  await page.goto(`/${ownerLogin}/${repoName}/actions`);
  await page.getByRole("link", { name: "API docs" }).click();
  await expect(page).toHaveURL(/\/docs\/api#actions-dashboard$/);
  await expect(
    page.getByRole("heading", { name: "Read Actions dashboard" }),
  ).toBeVisible();
  await page.goto(`/${ownerLogin}/${repoName}/actions`);
  await page
    .getByRole("link", { name: "Open run 1 details and options" })
    .click();
  await expect(page).toHaveURL(new RegExp(`/actions/runs/${run.id}`));
  await expect(
    page.getByRole("heading", { name: "Workflow run" }),
  ).toBeVisible();
  await page.goto(`/${ownerLogin}/${repoName}/actions`);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-001-phase4-management-docs.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-001-phase3-filters.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-001-phase2-actions-list.jpg",
  });

  const emptyRepository = await createRepository(
    page,
    `actions empty ${Date.now().toString(36)}`,
  );
  await page.goto(
    `/${emptyRepository.ownerLogin}/${emptyRepository.repoName}/actions`,
  );
  await expect(
    page.getByText("Start automating this repository"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New workflow" }).first(),
  ).toHaveAttribute("href", /\/new\/main\/.github\/workflows$/);
  await expect(
    page.getByText("Rust").locator("xpath=ancestor::a[1]"),
  ).toHaveAttribute("href", /\/new\/main\/.github\/workflows$/);
  await expectNoDeadControls(page);
});
