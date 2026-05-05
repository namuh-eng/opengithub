import { Buffer } from "node:buffer";
import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededDashboard = {
  cookieName: string;
  cookieValue: string;
  treeRepositoryHref: string;
};

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedDashboard(): SeededDashboard {
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
        DASHBOARD_E2E_SKIP_MIGRATIONS: "1",
        DASHBOARD_E2E_TREE_REFS: "1",
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededDashboard;
}

function seedDiscussions(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");

  execFileSync(
    "psql",
    [
      databaseUrl,
      "-v",
      "ON_ERROR_STOP=1",
      "-c",
      `
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
          repository_id, slug, name, emoji, description, position
        )
        SELECT target_repo.id, 'polls', 'Polls', '📊',
               'Collect structured feedback.', 4
        FROM target_repo
        ON CONFLICT (repository_id, slug)
        DO UPDATE SET name = EXCLUDED.name,
                      emoji = EXCLUDED.emoji,
                      description = EXCLUDED.description
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
      ON CONFLICT (discussion_id)
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
      `,
    ],
    { stdio: "ignore" },
  );
}

async function signIn(page: Page, seeded: SeededDashboard) {
  await page.context().addCookies([
    {
      domain: "localhost",
      httpOnly: true,
      name: seeded.cookieName,
      path: "/",
      sameSite: "Lax",
      secure: false,
      value: seeded.cookieValue,
    },
  ]);
}

async function expectNoDeadControls(page: Page) {
  await expect(page.locator('a[href="#"]')).toHaveCount(0);
  await expect(page.locator("button:not([type])")).toHaveCount(0);
}

test("repository discussions list filters, rows, category rail, and mobile layout work", async ({
  page,
}) => {
  const seeded = seedDashboard();
  seedDiscussions(seeded.treeRepositoryHref);
  await signIn(page, seeded);

  await page.goto(`${seeded.treeRepositoryHref}/discussions`);
  await expect(
    page.getByRole("heading", { name: "Discussions" }),
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
  await page
    .getByRole("link", { name: /repository import previews/i })
    .first()
    .click();
  await expect(page).toHaveURL(/\/discussions\/901$/);
  await expect(
    page.getByRole("heading", { name: /repository import previews/i }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Newest" })).toHaveAttribute(
    "href",
    /sort=newest/,
  );
  await expect(page.getByRole("button", { name: "Comment" })).toBeDisabled();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-003-phase2-detail.jpg",
  });

  await page.goto(`${seeded.treeRepositoryHref}/discussions`);
  await page.getByLabel("discussion-query").fill("manifest");
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page).toHaveURL(/\/discussions\?q=manifest/);

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

  await page.goto(`${seeded.treeRepositoryHref}/discussions/new/choose`);
  await expect(
    page.getByRole("heading", { name: "Choose a category" }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "General" })).toBeVisible();
  await expect(page.getByText("Answers enabled")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Get started" }).first(),
  ).toHaveAttribute("href", /\/discussions\/new\?category=general$/);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-phase2-chooser.jpg",
  });
  await page.getByRole("link", { name: "Get started" }).first().click();
  await expect(page).toHaveURL(/\/discussions\/new\?category=general$/);
  await expect(page.getByRole("heading", { name: /General/ })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Choose a different category" }),
  ).toHaveAttribute("href", /\/discussions\/new\/choose$/);
  await page
    .getByRole("textbox", { name: "Title" })
    .fill(`Search syntax ideas ${Date.now()}`);
  await page
    .getByLabel("Discussion body")
    .fill("Support saved discussion searches with **Markdown** preview.");
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByText("Markdown")).toBeVisible();
  await page.getByRole("tab", { name: "Write" }).click();
  await page.setInputFiles("input#discussion-attachments", {
    name: "sketch.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("discussion sketch"),
  });
  await expect(page.getByText("sketch.txt")).toBeVisible();
  await page
    .getByRole("checkbox", {
      name: /I have done a search for similar discussions/i,
    })
    .check();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-phase3-generic-create.jpg",
  });
  await page.getByRole("button", { name: "Start discussion" }).click();
  await expect(page).toHaveURL(/\/discussions\/903$/);

  await page.goto(`${seeded.treeRepositoryHref}/discussions/new?category=q-a`);
  await expect(page.getByText("Category form").first()).toBeVisible();
  await page
    .getByRole("textbox", { name: "Title" })
    .fill(`Template answer shape ${Date.now()}`);
  await page
    .getByLabel("Context")
    .fill("The form should persist category-specific context.");
  await page.getByLabel("Area").selectOption("API");
  await page
    .getByLabel("Discussion body")
    .fill("Question body stays separate from the template answers.");
  await page
    .getByRole("checkbox", {
      name: /I have done a search for similar discussions/i,
    })
    .check();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-phase4-yaml-form.jpg",
  });
  await page.getByRole("button", { name: "Start discussion" }).click();
  await expect(page).toHaveURL(/\/discussions\/904$/);

  await page.goto(
    `${seeded.treeRepositoryHref}/discussions/new?category=polls`,
  );
  await expect(page.getByText("Poll").first()).toBeVisible();
  await page
    .getByRole("textbox", { name: "Title" })
    .fill(`Branch policy poll ${Date.now()}`);
  await page
    .getByLabel("Question")
    .fill("Which branch policy should ship first?");
  await page.getByLabel("Poll option 1").fill("Linear history");
  await page.getByLabel("Poll option 2").fill("Required reviews");
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
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-phase4-form-poll.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-final-desktop.jpg",
  });
  await page.getByRole("button", { name: "Start discussion" }).click();
  await expect(page).toHaveURL(/\/discussions\/905$/);

  await page.goto(
    `${seeded.treeRepositoryHref}/discussions/categories/ideas?q=no-match`,
  );
  await expect(page.getByRole("heading", { name: /Ideas/ })).toBeVisible();
  await expect(
    page.getByText("No Ideas discussions match this view."),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New discussion" }).first(),
  ).toHaveAttribute("href", /\/discussions\/new\?category=ideas$/);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-001-final-desktop.jpg",
  });

  await page.goto(
    `${seeded.treeRepositoryHref}/discussions/categories/general`,
  );
  const upvote = page.getByRole("button", { name: "Upvote discussion 901" });
  await upvote.click();
  await expect(upvote).toHaveAttribute("aria-pressed", "true");
  await upvote.click();
  await expect(upvote).toHaveAttribute("aria-pressed", "false");

  await page.setViewportSize({ width: 390, height: 900 });
  await expect(page.locator("body")).not.toHaveCSS("overflow-x", "scroll");
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-001-final-mobile.jpg",
  });
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/discussions-002-final-mobile.jpg",
  });
});
