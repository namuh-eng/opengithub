import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  treeRepositoryHref?: string;
};

function seedSession(extraEnv: Record<string, string> = {}): SeededSession {
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
        ...extraEnv,
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

test.skip(
  !databaseUrl,
  "git transport E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("git docs and repository controls expose working HTTPS flows", async ({
  page,
}) => {
  const seeded = seedSession({ DASHBOARD_E2E_TREE_REFS: "1" });
  await signIn(page, seeded);
  if (!seeded.treeRepositoryHref) {
    throw new Error("tree repository seed did not return a repository href");
  }

  await page.goto("/docs/git");
  await expect(
    page.getByRole("heading", { name: "Work with repositories over HTTPS" }),
  ).toBeVisible();
  await expect(
    page.getByText(/git clone https:\/\/opengithub\.namuh\.co/).first(),
  ).toBeVisible();
  await expect(page.getByText("curl -L -o octo-app.zip")).toBeVisible();
  await expect(page.locator("article")).not.toContainText("api.github.com");
  await expectNoDeadControls(page);

  await page.goto(seeded.treeRepositoryHref);
  await page.locator("summary").filter({ hasText: "Code" }).click();
  await expect(page.getByLabel("HTTPS")).toHaveValue(/\.git$/);
  await expect(
    page.getByRole("link", { name: "Download ZIP" }),
  ).toHaveAttribute("href", /\/archive\/refs\/heads\/main\.zip$/);

  const rawResponse = await page.request.get(
    `${seeded.treeRepositoryHref}/raw/main/src/main.rs`,
  );
  expect(rawResponse.ok()).toBeTruthy();
  await expect(rawResponse.text()).resolves.toContain("tokio::main");

  const archiveResponse = await page.request.get(
    `${seeded.treeRepositoryHref}/archive/refs/heads/main.zip`,
  );
  expect(archiveResponse.ok()).toBeTruthy();
  expect(archiveResponse.headers()["content-type"]).toContain(
    "application/zip",
  );
  const archiveBytes = await archiveResponse.body();
  expect(archiveBytes.subarray(0, 2).toString()).toBe("PK");

  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/git-001-final-docs-code-menu.jpg",
  });
});

test("empty repository quick setup has copyable real Git commands", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `git quick setup ${Date.now().toString(36)}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page.getByLabel(/Description/).fill("Git quick setup guardrail");
  await page.getByRole("button", { name: "Create repository" }).click();

  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));
  await expect(
    page.getByRole("heading", { name: "Quick setup" }),
  ).toBeVisible();
  await expect(page.getByLabel("HTTPS clone URL")).toHaveValue(
    new RegExp(`/${normalizedName}\\.git$`),
  );
  await expect(page.getByText(/git clone/)).toBeVisible();
  await expect(page.getByText(/echo "# Getting started"/)).toBeVisible();
  await expect(page.getByText(/git push -u origin main/)).toBeVisible();
  await expect(page.getByRole("link", { name: "Git docs" })).toHaveAttribute(
    "href",
    "/docs/git",
  );
  await page.getByRole("button", { name: "Copy URL" }).click();
  await expect(page.getByRole("status")).toContainText(/copied|unavailable/);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/git-001-final-empty-quick-setup.jpg",
  });
});
