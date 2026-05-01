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
  await page.getByLabel(/Description/).fill("Workflow detail smoke testing");
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
  "repository Actions workflow E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in workflow Actions page renders scoped runs and filters", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `actions workflow ${Date.now().toString(36)}`;
  const { ownerLogin, repoName } = await createRepository(page, repositoryName);
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;

  const workflowPath = ".github/workflows/editorial-ci.yml";
  const workflowResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/actions/workflows`,
    {
      headers: { cookie },
      data: {
        name: "Editorial CI",
        path: workflowPath,
        triggerEvents: ["push", "workflow_dispatch"],
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
        event: "workflow_dispatch",
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

  await page.goto(
    `/${ownerLogin}/${repoName}/actions/workflows/${workflowPath}`,
  );
  await expect(
    page.getByRole("heading", { exact: true, name: "Editorial CI" }),
  ).toBeVisible();
  const workflowNav = page.getByRole("navigation", {
    name: "Actions workflows",
  });
  await expect(
    workflowNav.getByRole("link", { name: /Editorial CI/ }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.getByRole("button", { name: "Run workflow" })).toHaveCount(
    0,
  );
  await expect(
    page.getByRole("link", { exact: true, name: "Editorial CI" }).last(),
  ).toHaveAttribute("href", new RegExp(`/actions/runs/${run.id}`));
  await expect(page.getByLabel("Success run")).toBeVisible();
  await expect(page.getByRole("button", { name: "Workflow" })).toHaveCount(0);
  for (const filter of ["Event", "Status", "Branch", "Actor"]) {
    await expect(page.getByRole("button", { name: filter })).toBeVisible();
  }

  await page.getByPlaceholder("Filter this workflow's runs").fill("manual");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/q=manual/);
  await page.getByRole("button", { name: "Status" }).click();
  await page.getByRole("menuitemradio", { name: /success/i }).click();
  await expect(page).toHaveURL(/status=success/);
  await page.getByRole("button", { name: "Branch" }).click();
  await page.getByRole("menuitemradio", { name: /main/i }).click();
  await expect(page).toHaveURL(/branch=main/);
  await expectNoDeadControls(page);

  const desktopOverflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth >
      document.documentElement.clientWidth,
  );
  expect(desktopOverflow).toBe(false);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/actions-002-phase2-workflow-page.jpg",
  });
});
