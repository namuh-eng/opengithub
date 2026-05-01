import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededSession = {
  cookieName: string;
  cookieValue: string;
  firstRepositoryHref?: string;
};

type CreatedPullRequest = {
  number?: number;
  pull_request?: {
    number?: number;
  };
};

function seedSession({
  empty = true,
}: {
  empty?: boolean;
} = {}): SeededSession {
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
        DASHBOARD_E2E_EMPTY: empty ? "1" : "0",
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

test.skip(
  !databaseUrl,
  "repository pull request detail E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in user opens the pull request detail conversation shell", async ({
  page,
}) => {
  const seeded = seedSession();
  await signIn(page, seeded);
  const repositoryName = `detail repo ${Date.now().toString(36)}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page
    .getByLabel(/Description/)
    .fill("Repository for pull request detail smoke testing");
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));

  const [, ownerLogin, repoName] = new URL(page.url()).pathname.split("/");
  const pullTitle = `Detail read smoke ${Date.now().toString(36)}`;
  const createResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/pulls`,
    {
      headers: {
        cookie: `${seeded.cookieName}=${seeded.cookieValue}`,
      },
      data: {
        title: pullTitle,
        body: "Renders the **pull request detail** conversation shell.",
        headRef: "feature/detail-read",
        baseRef: "main",
        isDraft: true,
      },
    },
  );
  expect(createResponse.status()).toBe(201);
  const created = (await createResponse.json()) as CreatedPullRequest;
  const pullNumber = created.number ?? created.pull_request?.number;
  expect(pullNumber).toBeTruthy();

  await page.goto(`/${ownerLogin}/${repoName}/pull/${pullNumber}`);
  await expect(
    page.getByRole("heading", { name: new RegExp(pullTitle) }),
  ).toBeVisible();
  await expect(page.getByText("Draft", { exact: true })).toBeVisible();
  await expect(page.getByText(/wants to merge/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: /^Conversation/ }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { name: "Files changed" }),
  ).toHaveAttribute(
    "href",
    `/${ownerLogin}/${repoName}/pull/${pullNumber}/files`,
  );
  await page.getByRole("link", { name: "Files changed" }).click();
  await expect(
    page.getByRole("heading", { name: /Files changed/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("textbox", { name: "File filter" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Review changes" }),
  ).toBeDisabled();
  await expect(page.getByRole("link", { name: "Split" })).toHaveAttribute(
    "href",
    `/${ownerLogin}/${repoName}/pull/${pullNumber}/files?view=split`,
  );
  const inlineCommentButton = page
    .getByRole("button", { name: /Add comment at diff position/ })
    .first();
  if (await inlineCommentButton.isVisible()) {
    await inlineCommentButton.click();
    await page
      .getByRole("textbox", { name: /Pending review comment/ })
      .fill("Phase 3 pending **review** draft.");
    await page.getByRole("tab", { name: "Preview" }).click();
    await expect(page.getByText("Phase 3 pending")).toBeVisible();
    await page.getByRole("button", { name: "Save pending comment" }).click();
    await expect(
      page.getByText("left a pending review comment"),
    ).toBeVisible();
    await page.reload();
    await expect(
      page.getByText("left a pending review comment"),
    ).toBeVisible();
  }
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/prs-005-phase3-pending-comment.jpg",
  });

  await page.goto(`/${ownerLogin}/${repoName}/pull/${pullNumber}`);
  await expect(
    page.getByRole("heading", { name: "Merge readiness" }),
  ).toBeVisible();
  await expect(page.getByText("No review requests")).toBeVisible();
  await expect(
    page.getByText(new RegExp(`${ownerLogin} opened this pull request`)),
  ).toBeVisible();

  await page
    .getByRole("textbox", { name: "Comment body" })
    .fill("Phase 2 browser **comment** works.");
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByText(/Phase 2 browser/)).toBeVisible();
  await page.getByRole("tab", { name: "Write" }).click();
  await page.getByRole("button", { name: "Comment" }).click();
  await expect(page.getByText("Comment posted.")).toBeVisible();
  await expect(page.getByText("Phase 2 browser")).toBeVisible();
  await page.reload();
  await expect(page.getByText("Phase 2 browser")).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/prs-004-phase2-comment.jpg",
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/${ownerLogin}/${repoName}/pull/${pullNumber}`);
  await expect(
    page.getByRole("heading", { name: new RegExp(pullTitle) }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Merge readiness" }),
  ).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/prs-004-phase5-mobile-detail.jpg",
  });
});

test("signed-in user updates pull request sidebar metadata and notifications", async ({
  page,
}) => {
  const seeded = seedSession({ empty: false });
  await signIn(page, seeded);
  const repositoryHref = seeded.firstRepositoryHref;
  if (!repositoryHref) {
    throw new Error("dashboard seed did not return a repository href");
  }
  const [, ownerLogin, repoName] = repositoryHref.split("/");
  const pullTitle = `Phase 3 sidebar smoke ${Date.now().toString(36)}`;
  const createResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/pulls`,
    {
      headers: {
        cookie: `${seeded.cookieName}=${seeded.cookieValue}`,
      },
      data: {
        title: pullTitle,
        body: "Exercises review requests, labels, draft state, and notifications.",
        headRef: "feature/sidebar-actions",
        baseRef: "main",
        isDraft: false,
      },
    },
  );
  expect(createResponse.status()).toBe(201);
  const created = (await createResponse.json()) as CreatedPullRequest;
  const pullNumber = created.number ?? created.pull_request?.number;
  expect(pullNumber).toBeTruthy();

  await page.goto(`${repositoryHref}/pull/${pullNumber}`);
  await expect(
    page.getByRole("heading", { name: new RegExp(pullTitle) }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Edit" }).first().click();
  await page.getByRole("button", { name: /^Request reviewer-/ }).click();
  await expect(page.getByText("Review requests updated.")).toBeVisible();
  await expect(page.getByText("requested", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Edit" }).nth(2).click();
  await page.getByRole("button", { name: /Add bug/ }).click();
  await expect(page.getByText("Pull request metadata updated.")).toBeVisible();
  await expect(page.getByText("bug")).toBeVisible();

  await page.getByRole("button", { name: "Convert to draft" }).click();
  await expect(
    page.getByText("Pull request converted to draft."),
  ).toBeVisible();
  await expect(page.getByText("Draft", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Mark ready" }).first().click();
  await expect(
    page.getByText("Pull request marked ready for review."),
  ).toBeVisible();
  await expect(
    page.getByText("There are no changed files or commits to merge.").first(),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Merge pull request" }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Close pull request" }).click();
  await expect(page.getByText("Pull request closed.")).toBeVisible();
  await expect(page.getByText("Closed", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Reopen pull request" }).click();
  await expect(page.getByText("Pull request reopened.")).toBeVisible();
  await expect(page.getByText("Open", { exact: true })).toBeVisible();

  const unsubscribeButton = page.getByRole("button", {
    exact: true,
    name: "Unsubscribe",
  });
  if (await unsubscribeButton.isVisible()) {
    await unsubscribeButton.click();
    await expect(page.getByText("Unsubscribed.")).toBeVisible();
    await expect(page.getByText("Not subscribed")).toBeVisible();
  }
  await page.getByRole("button", { exact: true, name: "Subscribe" }).click();
  await expect(page.getByText("Subscribed to notifications.")).toBeVisible();
  await expect(page.getByText("Subscribed: subscribed")).toBeVisible();

  await page.reload();
  await expect(page.getByText("Open", { exact: true })).toBeVisible();
  await expect(page.getByText("bug")).toBeVisible();
  await expect(page.getByText("Subscribed: subscribed")).toBeVisible();
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);

  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/prs-004-phase3-sidebar-actions.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/prs-004-phase4-mergeability.jpg",
  });
});
