import { Buffer } from "node:buffer";
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import {
  expect,
  expectNoDeadControls,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function runSql(sql: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }

  try {
    execFileSync("psql", [databaseUrl, "-v", "ON_ERROR_STOP=1", "-c", sql], {
      stdio: "ignore",
    });
    return;
  } catch (error) {
    const nodeError = error as NodeJS.ErrnoException;
    if (nodeError.code !== "ENOENT") {
      throw error;
    }
  }

  const parsed = new URL(databaseUrl);
  const runtime = existsSync("/usr/bin/podman") ? "podman" : "docker";
  execFileSync(
    runtime,
    [
      "exec",
      "-e",
      `PGPASSWORD=${decodeURIComponent(parsed.password)}`,
      "opengithub-postgres-test",
      "psql",
      "-U",
      decodeURIComponent(parsed.username),
      "-d",
      decodeURIComponent(parsed.pathname.slice(1)),
      "-v",
      "ON_ERROR_STOP=1",
      "-c",
      sql,
    ],
    { stdio: "ignore" },
  );
}

async function expectApiSessionReady(cookieName: string, cookieValue: string) {
  await expect
    .poll(
      async () => {
        const response = await fetch("http://localhost:3016/api/auth/me", {
          headers: { cookie: `${cookieName}=${cookieValue}` },
        }).catch(() => null);
        return response?.status ?? 0;
      },
      { timeout: 30_000 },
    )
    .toBe(200);
}

function seedDiscussions(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");

  runSql(`
      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      general AS (
        INSERT INTO discussion_categories (
          repository_id, slug, name, emoji, description, position
        )
        SELECT target_repo.id, 'general', 'General', '💬',
               'General project conversation.', 1
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name,
                      emoji = EXCLUDED.emoji,
                      description = EXCLUDED.description
        RETURNING id, repository_id
      ),
      ideas AS (
        INSERT INTO discussion_categories (
          repository_id, slug, name, emoji, description, position
        )
        SELECT target_repo.id, 'ideas', 'Ideas', '💡',
               'Shape product direction.', 2
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name,
                      emoji = EXCLUDED.emoji,
                      description = EXCLUDED.description
        RETURNING id, repository_id
      ),
      qa AS (
        INSERT INTO discussion_categories (
          repository_id, slug, name, emoji, description, position, accepts_answers
        )
        SELECT target_repo.id, 'q-a', 'Q&A', '🙏',
               'Ask an answerable question.', 3, true
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name,
                      emoji = EXCLUDED.emoji,
                      description = EXCLUDED.description,
                      accepts_answers = true
        RETURNING id, repository_id
      ),
      polls AS (
        INSERT INTO discussion_categories (
          repository_id, slug, name, emoji, description, position, format
        )
        SELECT target_repo.id, 'polls', 'Polls', '📊',
               'Collect structured feedback.', 4, 'poll'
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name,
                      emoji = EXCLUDED.emoji,
                      description = EXCLUDED.description,
                      format = 'poll',
                      accepts_answers = false
        RETURNING id, repository_id
      ),
      label_one AS (
        INSERT INTO labels (repository_id, name, color, description)
        SELECT target_repo.id, 'help-wanted', 'b85f36', 'Needs community input'
        FROM target_repo
        ON CONFLICT (repository_id, lower(name))
        DO UPDATE SET description = EXCLUDED.description
        RETURNING id, repository_id
      ),
      discussion_one AS (
        INSERT INTO discussions (
          repository_id, category_id, number, title, body, state, answered,
          author_user_id, comments_count, votes_count, last_activity_at
        )
        SELECT general.repository_id,
               general.id,
               901,
               'How should repository import previews handle large manifests ${suffix}?',
               'Discuss manifest previews.',
               'open',
               true,
               target_repo.author_user_id,
               8,
               14,
               now() - interval '1 hour'
        FROM general, target_repo
        ON CONFLICT (repository_id, number)
        DO UPDATE SET title = EXCLUDED.title,
                      answered = true,
                      comments_count = 8,
                      votes_count = 14,
                      last_activity_at = EXCLUDED.last_activity_at
        RETURNING id, repository_id
      ),
      discussion_two AS (
        INSERT INTO discussions (
          repository_id, category_id, number, title, body, state,
          author_user_id, comments_count, votes_count, last_activity_at
        )
        SELECT ideas.repository_id,
               ideas.id,
               902,
               'Empty-state CTA should preserve category context ${suffix}',
               'Discuss category route behavior.',
               'open',
               target_repo.author_user_id,
               1,
               3,
               now() - interval '2 hours'
        FROM ideas, target_repo
        ON CONFLICT (repository_id, number)
        DO UPDATE SET title = EXCLUDED.title,
                      comments_count = 1,
                      votes_count = 3,
                      last_activity_at = EXCLUDED.last_activity_at
        RETURNING id, repository_id
      ),
      comment_one AS (
        INSERT INTO discussion_comments (discussion_id, author_user_id, body)
        SELECT discussion_one.id, target_repo.author_user_id, 'Useful answer'
        FROM discussion_one, target_repo
        RETURNING id, discussion_id
      )
      INSERT INTO discussion_labels (discussion_id, label_id)
      SELECT discussion_one.id, label_one.id
      FROM discussion_one, label_one
      ON CONFLICT DO NOTHING;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      qa AS (
        SELECT discussion_categories.id, discussion_categories.repository_id
        FROM discussion_categories, target_repo
        WHERE discussion_categories.repository_id = target_repo.id
          AND discussion_categories.slug = 'q-a'
        LIMIT 1
      )
      INSERT INTO discussion_category_forms (
        repository_id, category_id, template_path, title, description, body, fields, valid
      )
      SELECT qa.repository_id,
             qa.id,
             '.github/DISCUSSION_TEMPLATE/q-a.yml',
             'Ask a question',
             'Add enough context for a maintainer to answer.',
             '',
             '[{"id":"context","fieldType":"textarea","label":"Context","description":"Tell maintainers what you tried.","placeholder":"What should happen?","required":true,"options":[]},{"id":"area","fieldType":"dropdown","label":"Area","description":null,"placeholder":null,"required":false,"options":["UI","API"]}]'::jsonb,
             true
      FROM qa
      ON CONFLICT (repository_id, category_id)
      DO UPDATE SET title = EXCLUDED.title,
                    description = EXCLUDED.description,
                    fields = EXCLUDED.fields,
                    valid = true,
                    updated_at = now();

      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      discussion_one AS (
        SELECT discussions.id
        FROM discussions, target_repo
        WHERE discussions.repository_id = target_repo.id
          AND discussions.number = 901
        LIMIT 1
      )
      INSERT INTO discussion_pins (discussion_id, pinned_by_user_id, position)
      SELECT discussion_one.id, target_repo.author_user_id, 1
      FROM discussion_one, target_repo
      ON CONFLICT (discussion_id) WHERE (pin_scope = 'global')
      DO UPDATE SET position = EXCLUDED.position;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_community_links (repository_id, label, href, kind, position)
      SELECT target_repo.id,
             'Code of conduct',
             ${sqlLiteral(`${repositoryHref}/community/code-of-conduct`)},
             'code_of_conduct',
             1
      FROM target_repo
      ON CONFLICT DO NOTHING;
      `);
}

function assertCreatedDiscussion(
  repositoryHref: string,
  title: string,
  options: {
    formAnswerCount?: number;
    pollOptionCount?: number;
  } = {},
) {
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const formAnswerCount = options.formAnswerCount ?? 0;
  const pollOptionCount = options.pollOptionCount ?? 0;

  runSql(`
    DO $$
    DECLARE
      created_discussion_id uuid;
      actual_form_answers bigint;
      actual_poll_options bigint;
      actual_comments bigint;
      actual_subscriptions bigint;
    BEGIN
      SELECT discussions.id
      INTO created_discussion_id
      FROM discussions
      JOIN repositories ON repositories.id = discussions.repository_id
      LEFT JOIN users ON users.id = repositories.owner_user_id
      LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
      WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
        AND repositories.name = ${sqlLiteral(decodedRepo)}
        AND discussions.title = ${sqlLiteral(title)}
      ORDER BY discussions.created_at DESC
      LIMIT 1;

      IF created_discussion_id IS NULL THEN
        RAISE EXCEPTION 'discussion % was not created', ${sqlLiteral(title)};
      END IF;

      SELECT COUNT(*) INTO actual_comments
      FROM discussion_comments
      WHERE discussion_id = created_discussion_id;
      IF actual_comments < 1 THEN
        RAISE EXCEPTION 'discussion % did not create opening body comment', ${sqlLiteral(title)};
      END IF;

      SELECT COUNT(*) INTO actual_subscriptions
      FROM discussion_subscriptions
      WHERE discussion_id = created_discussion_id
        AND state = 'subscribed'
        AND reason = 'participating';
      IF actual_subscriptions < 1 THEN
        RAISE EXCEPTION 'discussion % did not create participant subscription', ${sqlLiteral(title)};
      END IF;

      SELECT COUNT(*) INTO actual_form_answers
      FROM discussion_form_answers
      WHERE discussion_id = created_discussion_id;
      IF actual_form_answers != ${formAnswerCount} THEN
        RAISE EXCEPTION 'discussion % expected % form answers, got %',
          ${sqlLiteral(title)}, ${formAnswerCount}, actual_form_answers;
      END IF;

      SELECT COUNT(discussion_poll_options.id) INTO actual_poll_options
      FROM discussion_polls
      JOIN discussion_poll_options ON discussion_poll_options.poll_id = discussion_polls.id
      WHERE discussion_polls.discussion_id = created_discussion_id;
      IF actual_poll_options != ${pollOptionCount} THEN
        RAISE EXCEPTION 'discussion % expected % poll options, got %',
          ${sqlLiteral(title)}, ${pollOptionCount}, actual_poll_options;
      END IF;
    END $$;
  `);
}

function seedConvertibleIssue(repositoryHref: string): number {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const issueNumber = 970 + Math.floor(Math.random() * 20);

  runSql(`
      WITH target_repo AS (
        SELECT repositories.id, users.id AS author_user_id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      issue AS (
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        SELECT target_repo.id,
               ${issueNumber},
               'Convert this issue into a discussion',
               'Issue body copied into the converted discussion.',
               'open',
               target_repo.author_user_id
        FROM target_repo
        ON CONFLICT (repository_id, number)
        DO UPDATE SET title = EXCLUDED.title,
                      body = EXCLUDED.body,
                      state = 'open',
                      converted_discussion_id = NULL,
                      converted_to_discussion_at = NULL,
                      converted_to_discussion_by_user_id = NULL
        RETURNING id, repository_id, author_user_id
      )
      INSERT INTO comments (repository_id, issue_id, author_user_id, body)
      SELECT issue.repository_id,
             issue.id,
             issue.author_user_id,
             'Issue comment copied during conversion.'
      FROM issue;
      `);
  return issueNumber;
}

test.skip(
  skipWithoutTestDb(),
  "repository discussions smoke needs a database URL",
);
test.setTimeout(120_000);
test("repository discussions list filters, category rail, empty state, and upvotes work", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDiscussions(repositoryHref);
  await signIn(page, seeded, "owner");
  await expectApiSessionReady(seeded.cookieName, seeded.cookies.owner);

  await page.goto(`${repositoryHref}/discussions`);
  await expect(
    page.getByRole("heading", { name: "Discussions", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Discussions" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    page.getByRole("region", { name: "Pinned discussions" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /repository import previews/i }).first(),
  ).toBeVisible();
  await expect(page.getByLabel("discussion-query")).toHaveValue("is:open");
  await expect(page.getByRole("button", { name: "Search" })).toBeVisible();
  await expect(page.getByText("Sort: latest", { exact: true })).toBeVisible();
  await page.getByText("Sort: Latest activity").click();
  await expect(page.getByRole("link", { name: "Newest" })).toHaveAttribute(
    "href",
    /sort=newest/,
  );
  await expect(
    page.getByRole("link", { name: /help-wanted/ }).first(),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Categories" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: /General/ }).last(),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Most helpful" }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Community" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Code of conduct" }),
  ).toBeVisible();

  await page.getByLabel("discussion-query").fill("manifest");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/\/discussions\?q=manifest/);
  await expect(
    page.getByRole("link", { name: /repository import previews/i }).first(),
  ).toBeVisible();

  await page
    .getByRole("link", { name: /General/ })
    .last()
    .click();
  await expect(page).toHaveURL(/\/discussions\/categories\/general/);
  await expect(page.getByRole("heading", { name: /General/ })).toBeVisible();
  await expect(page.getByText("category:general")).toBeVisible();
  await expect(
    page.getByRole("link", { name: /General.*active category/ }),
  ).toHaveAttribute("aria-current", "page");

  await page.goto(`${repositoryHref}/discussions/categories/ideas?q=no-match`);
  await expect(page.getByRole("heading", { name: "💡 Ideas" })).toBeVisible();
  await expect(
    page.getByText("No Ideas discussions match this view."),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New discussion" }).first(),
  ).toHaveAttribute("href", /\/discussions\/new\?category=ideas$/);

  await page.goto(`${repositoryHref}/discussions/categories/general`);
  await page.getByRole("button", { name: "Upvote discussion 901" }).click();
  const removeUpvote = page.getByRole("button", {
    name: "Remove upvote from discussion 901",
  });
  await expect(removeUpvote).toHaveAttribute("aria-pressed", "true");
  await removeUpvote.click();
  await expect(
    page.getByRole("button", { name: "Upvote discussion 901" }),
  ).toHaveAttribute("aria-pressed", "false");
  await expectNoDeadControls(page);
});

test("repository discussion creation supports chooser, YAML form validation, Markdown preview, acknowledgement, and redirect", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDiscussions(repositoryHref);
  await signIn(page, seeded, "owner");
  await expectApiSessionReady(seeded.cookieName, seeded.cookies.owner);

  await page.goto(`${repositoryHref}/discussions/new/choose`);
  await expect(
    page.getByRole("heading", { name: "Choose a category" }),
  ).toBeVisible();
  const qaCard = page
    .locator("article")
    .filter({ has: page.getByRole("heading", { name: "Q&A" }) });
  await expect(qaCard.getByText("🙏")).toBeVisible();
  await expect(qaCard.getByText("Answers enabled")).toBeVisible();
  await qaCard.getByRole("link", { name: "Get started" }).click();

  await expect(page).toHaveURL(/\/discussions\/new\?category=q-a$/);
  await expect(page.getByRole("heading", { name: /Q&A/ })).toBeVisible();
  await expect(
    page.getByText("Category form", { exact: true }).first(),
  ).toBeVisible();
  await expect(page.getByLabel("Context *")).toBeVisible();
  await expect(page.getByText("Context is required.")).toBeVisible();

  const title = `QA discussion creation ${Date.now()}`;
  await page.getByLabel("Title *").fill(title);
  await expect(
    page.getByRole("link", { name: "Search using this title" }),
  ).toHaveAttribute(
    "href",
    new RegExp(`/discussions\\?q=is%3Aopen\\+${title.replaceAll(" ", "\\+")}`),
  );
  await page
    .getByRole("textbox", { name: "Discussion body" })
    .fill("**Preview this body**");
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByText("Preview this body")).toBeVisible();
  await expect(page).toHaveURL(/\/discussions\/new\?category=q-a$/);

  await page.getByRole("tab", { name: "Write" }).click();
  await page.getByLabel("Context *").fill("Maintainers need a real workflow.");
  await page.getByLabel("Area").selectOption("UI");
  await page
    .getByRole("checkbox", {
      name: /I have done a search for similar discussions/i,
    })
    .focus();
  await page.keyboard.press("Tab");
  await expect(
    page.getByText("Similar-search acknowledgement is required."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Start discussion" }),
  ).toBeDisabled();

  await page
    .getByRole("checkbox", {
      name: /I have done a search for similar discussions/i,
    })
    .check();
  await page.getByRole("button", { name: "Start discussion" }).click();
  await expect(page).toHaveURL(/\/discussions\/\d+$/);
  await expect(page.getByRole("heading", { name: title })).toBeVisible();
  assertCreatedDiscussion(repositoryHref, title, { formAnswerCount: 2 });
  await expectNoDeadControls(page);
});

test("repository discussion creation supports poll category question and options", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDiscussions(repositoryHref);
  await signIn(page, seeded, "owner");
  await expectApiSessionReady(seeded.cookieName, seeded.cookies.owner);

  await page.goto(`${repositoryHref}/discussions/new/choose`);
  const pollCard = page
    .locator("article")
    .filter({ has: page.getByRole("heading", { name: "Polls" }) });
  await expect(pollCard.getByText("📊")).toBeVisible();
  await expect(pollCard.getByText("Poll", { exact: true })).toBeVisible();
  await pollCard.getByRole("link", { name: "Get started" }).click();

  await expect(page).toHaveURL(/\/discussions\/new\?category=polls$/);
  await expect(page.getByRole("heading", { name: /Polls/ })).toBeVisible();
  await expect(page.getByText("Poll", { exact: true }).first()).toBeVisible();
  await expect(page.getByLabel("Question *")).toBeVisible();
  await expect(page.getByLabel("Poll option 1")).toBeVisible();
  await expect(page.getByLabel("Context *")).toHaveCount(0);

  const title = `QA poll creation ${Date.now()}`;
  await page.getByLabel("Title *").fill(title);
  await page
    .getByLabel("Question *")
    .fill("Which discussion creation path should ship first?");
  await page.getByLabel("Poll option 1").fill("YAML forms");
  await page.getByLabel("Poll option 2").fill("Polls");
  await page
    .getByRole("checkbox", {
      name: /Allow voters to choose more than one option/i,
    })
    .check();
  await page
    .getByRole("checkbox", {
      name: /I have done a search for similar discussions/i,
    })
    .check();
  await page.getByRole("button", { name: "Start discussion" }).click();

  await expect(page).toHaveURL(/\/discussions\/\d+$/);
  await expect(page.getByRole("heading", { name: title })).toBeVisible();
  assertCreatedDiscussion(repositoryHref, title, { pollOptionCount: 2 });
  await expectNoDeadControls(page);
});
test("repository issue converts into a discussion from the issue sidebar", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const repositoryHref = seeded.hrefs.treeRepository;
  seedDiscussions(repositoryHref);
  const issueNumber = seedConvertibleIssue(repositoryHref);
  await signIn(page, seeded, "owner");
  await expectApiSessionReady(seeded.cookieName, seeded.cookies.owner);

  await page.goto(`${repositoryHref}/issues/${issueNumber}`);
  await expect(
    page.getByRole("heading", { name: /Convert this issue into a discussion/ }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Convert to discussion" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByText(/1 issue comments will be copied/)).toBeVisible();
  await page.getByLabel("Discussion category").selectOption("general");
  await page.getByRole("button", { name: "Convert issue" }).click();
  await expect(page).toHaveURL(/\/discussions\/\d+$/);
  await expect(
    page.getByRole("heading", { name: /Convert this issue into a discussion/ }),
  ).toBeVisible();
  await expect(page.getByText(/converted from issue/i)).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-005-phase4-issue-conversion.jpg",
  });
  await expectNoDeadControls(page);
});
