import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  runPsql,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type CreatedIssue = {
  number: number;
};

type CreatedPullRequest = {
  number?: number;
  pull_request?: { number: number };
};

type CreatedLabel = {
  label: {
    id: string;
    name: string;
  };
};

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedDiscussion(repositoryHref: string, labelId: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const discussionNumber = 980 + Math.floor(Math.random() * 10);

  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
          AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))}
        LIMIT 1
      ),
      category AS (
        INSERT INTO discussion_categories (
          repository_id, slug, name, emoji, description, position
        )
        SELECT target_repo.id, 'general', 'General', 'chat',
               'Repository label management discussion.', 1
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name
        RETURNING id, repository_id
      ),
      discussion AS (
        INSERT INTO discussions (
          repository_id, category_id, number, title, body, state,
          author_user_id, comments_count, votes_count, last_activity_at
        )
        SELECT category.repository_id,
               category.id,
               ${discussionNumber},
               'Label consistency sweep',
               'Labels should stay consistent across Discussions.',
               'open',
               target_repo.author_user_id,
               0,
               0,
               now()
        FROM category, target_repo
        RETURNING id
      )
      INSERT INTO discussion_labels (discussion_id, label_id)
      SELECT discussion.id, ${sqlLiteral(labelId)}::uuid
      FROM discussion
      ON CONFLICT DO NOTHING;
      `,
  ]);

  return discussionNumber;
}

test.skip(
  skipWithoutTestDb(),
  "Repository labels E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in repository labels manage and apply across issues, pull requests, and discussions", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["empty"] });
  await signIn(page, seeded, "owner");
  const unique = Date.now().toString(36);
  const repositoryName = `labels sweep ${unique}`;
  const normalizedName = repositoryName.replaceAll(/\s+/g, "-");

  await page.goto("/new");
  await page.getByLabel("Repository name *").fill(repositoryName);
  await page
    .getByLabel(/Description/)
    .fill("Repository for label management E2E coverage");
  await page.getByRole("button", { name: "Create repository" }).click();
  await expect(page).toHaveURL(new RegExp(`/${normalizedName}$`));

  const [, ownerLogin, repoName] = new URL(page.url()).pathname.split("/");
  const cookie = `${seeded.cookieName}=${seeded.cookies.owner}`;
  const labelResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/labels`,
    {
      headers: { cookie },
      data: {
        name: `triage ${unique}`,
        description: "Needs label QA",
        color: "b85c38",
      },
    },
  );
  expect(labelResponse.status()).toBe(201);
  const label = (await labelResponse.json()) as CreatedLabel;

  const issueResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/issues`,
    {
      headers: { cookie },
      data: {
        title: `Label issue ${unique}`,
        body: "Issue label assignment target.",
      },
    },
  );
  expect(issueResponse.status()).toBe(201);
  const issue = (await issueResponse.json()) as CreatedIssue;

  const pullResponse = await page.request.post(
    `http://localhost:3016/api/repos/${ownerLogin}/${repoName}/pulls`,
    {
      headers: { cookie },
      data: {
        title: `Label pull ${unique}`,
        body: "Pull request label assignment target.",
        headRef: "feature/sidebar-actions",
        baseRef: "main",
        isDraft: false,
      },
    },
  );
  expect(pullResponse.status()).toBe(201);
  const pull = (await pullResponse.json()) as CreatedPullRequest;
  const pullNumber = pull.number ?? pull.pull_request?.number;
  expect(pullNumber).toBeTruthy();
  const discussionNumber = seedDiscussion(
    `/${ownerLogin}/${repoName}`,
    label.label.id,
  );

  await page.goto(`/${ownerLogin}/${repoName}/labels`);
  await expect(
    page.getByRole("heading", { name: "Labels", exact: true }),
  ).toBeVisible();
  await page
    .getByPlaceholder("Search all labels")
    .fill(label.label.name.slice(0, 6));
  await expect(page.getByText(label.label.name)).toBeVisible();
  await page.getByRole("button", { name: "Sort" }).click();
  await expect(
    page.getByRole("menuitemradio", { name: /Total issue count/ }),
  ).toHaveAttribute("href", /sort=total_issue_count/);

  await page.getByRole("button", { name: "New label" }).click();
  await page.getByLabel("Label name").fill(`docs ${unique}`);
  await page.getByLabel("Label description").fill("Documentation follow-up");
  await page.getByLabel("Label color").fill("7f6a42");
  await page.getByRole("button", { name: "Save label" }).click();
  await expect(page.getByText("Label created.")).toBeVisible();
  await page.getByRole("button", { name: "Edit" }).first().click();
  await page.getByLabel("Label description").fill("Reviewed by E2E");
  await page.getByRole("button", { name: "Save label" }).click();
  await expect(page.getByText("Label updated.")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/labels-001-final-desktop.jpg",
  });

  await page.goto(`/${ownerLogin}/${repoName}/issues/${issue.number}`);
  await page
    .locator("section", {
      has: page.getByRole("heading", { name: "Labels", exact: true }),
    })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Search labels").fill(label.label.name.slice(0, 6));
  await page
    .getByRole("checkbox", { name: new RegExp(label.label.name) })
    .check();
  await page.getByRole("button", { name: "Save labels" }).click();
  await expect(page.getByText("Issue metadata updated.")).toBeVisible();
  await expect(
    page.locator(".chip", { hasText: label.label.name }),
  ).toBeVisible();

  await page.goto(`/${ownerLogin}/${repoName}/pull/${pullNumber}`);
  await page
    .locator("section", {
      has: page.getByRole("heading", { name: "Labels", exact: true }),
    })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Search labels").fill(label.label.name.slice(0, 6));
  await page
    .getByRole("checkbox", { name: new RegExp(label.label.name) })
    .check();
  await page.getByRole("button", { name: "Save labels" }).click();
  await expect(page.getByText("Pull request metadata updated.")).toBeVisible();
  await expect(
    page.locator(".chip", { hasText: label.label.name }),
  ).toBeVisible();

  await page.goto(`/${ownerLogin}/${repoName}/discussions/${discussionNumber}`);
  await expect(
    page.getByRole("heading", { name: "Label consistency sweep" }),
  ).toBeVisible();
  await expect(
    page.locator(".chip", { hasText: label.label.name }),
  ).toBeVisible();
  await page
    .locator("section", {
      has: page.getByRole("heading", { name: "Labels", exact: true }),
    })
    .getByRole("button", { name: "Edit" })
    .click();
  await page.getByLabel("Search labels").fill("docs");
  await page
    .getByRole("checkbox", { name: new RegExp(`docs ${unique}`) })
    .check();
  await page.getByRole("button", { name: "Save labels" }).click();
  await expect(page.getByText("Discussion metadata updated.")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  await page.goto(`/${ownerLogin}/${repoName}/labels`);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Labels", exact: true }),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/labels-001-final-mobile.jpg",
  });
});
