import { execFileSync } from "node:child_process";
import { expect, type Page, test } from "@playwright/test";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

type SeededWiki = {
  cookieName: string;
  cookieValue: string;
  repositoryWikiHref: string;
};

type WikiMutationResult = {
  page: {
    slug: string;
    href: string;
    revision?: { id: string; shortOid?: string };
  };
  redirectHref: string;
};

type WikiHistoryView = {
  revisions: Array<{
    id: string;
    message: string;
    shortOid: string | null;
    revisionHref: string;
  }>;
};

function seedWiki(): SeededWiki {
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
        SESSION_COOKIE_NAME: "og_session",
      },
    },
  ).toString();
  return JSON.parse(output) as SeededWiki;
}

async function signIn(page: Page, seeded: SeededWiki) {
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
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
}

async function waitForApiHealth(page: Page) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      const response = await page.request.get("http://localhost:3016/health", {
        timeout: 1000,
      });
      if (response.ok()) {
        return;
      }
    } catch {
      // make dev starts Next and Rust together; Next can win the readiness race.
    }
    await page.waitForTimeout(500);
  }
  throw new Error(
    "Rust API did not become healthy for repository wiki history E2E",
  );
}

function repositoryParts(repositoryWikiHref: string) {
  const [, owner, repo] = repositoryWikiHref.split("/");
  return { owner, repo };
}

async function createTwoRevisionPage(
  page: Page,
  seeded: SeededWiki,
  title: string,
) {
  const { owner, repo } = repositoryParts(seeded.repositoryWikiHref);
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const createResponse = await page.request.post(
    `http://localhost:3016/api/repos/${owner}/${repo}/wiki/pages`,
    {
      headers: { cookie },
      data: {
        title,
        markdown: `# ${title}\n\nOriginal history body.`,
        message: `Create ${title}`,
        editMode: "markdown",
      },
    },
  );
  expect(createResponse.status()).toBe(201);
  const created = (await createResponse.json()) as WikiMutationResult;
  const baseRevisionId = created.page.revision?.id;
  expect(baseRevisionId).toBeTruthy();

  const updateResponse = await page.request.patch(
    `http://localhost:3016/api/repos/${owner}/${repo}/wiki/${encodeURIComponent(
      created.page.slug,
    )}`,
    {
      headers: { cookie },
      data: {
        title,
        markdown: `# ${title}\n\nUpdated history body.`,
        message: `Update ${title}`,
        editMode: "markdown",
        expectedRevisionId: baseRevisionId,
      },
    },
  );
  expect(updateResponse.status()).toBe(200);
  const updated = (await updateResponse.json()) as WikiMutationResult;
  const headRevisionId = updated.page.revision?.id;
  expect(headRevisionId).toBeTruthy();

  return {
    owner,
    repo,
    cookie,
    slug: created.page.slug,
    baseRevisionId: baseRevisionId as string,
    headRevisionId: headRevisionId as string,
    pageHref: created.page.href,
  };
}

test.skip(
  !databaseUrl,
  "Repository Wiki history E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.setTimeout(90_000);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in repository wiki supports history, snapshots, compare, revert, and mobile layout", async ({
  page,
}) => {
  const seeded = seedWiki();
  await signIn(page, seeded);
  const title = `History Sweep ${Date.now().toString(36)}`;
  const fixture = await createTwoRevisionPage(page, seeded, title);

  await page.goto(
    `${seeded.repositoryWikiHref}/${encodeURIComponent(fixture.slug)}/_history`,
  );
  await expect(page.getByRole("heading", { name: "History" })).toBeVisible();
  await expect(page.getByText(`Create ${title}`)).toBeVisible();
  await expect(page.getByText(`Update ${title}`)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  const historyResponse = await page.request.get(
    `http://localhost:3016/api/repos/${fixture.owner}/${fixture.repo}/wiki/${encodeURIComponent(
      fixture.slug,
    )}/_history`,
    { headers: { cookie: fixture.cookie } },
  );
  expect(historyResponse.status()).toBe(200);
  const history = (await historyResponse.json()) as WikiHistoryView;
  const baseRevision = history.revisions.find((revision) =>
    revision.message.includes(`Create ${title}`),
  );
  const headRevision = history.revisions.find((revision) =>
    revision.message.includes(`Update ${title}`),
  );
  expect(baseRevision?.shortOid).toBeTruthy();
  expect(headRevision?.shortOid).toBeTruthy();

  await page.goto(baseRevision?.revisionHref ?? fixture.pageHref);
  await expect(
    page.getByRole("heading", { name: "Historical wiki revision" }),
  ).toBeVisible();
  await expect(page.getByText("Original history body.")).toBeVisible();

  await page.goto(
    `${seeded.repositoryWikiHref}/_compare?base=${fixture.baseRevisionId}&head=${fixture.headRevisionId}&page=${encodeURIComponent(
      fixture.slug,
    )}`,
  );
  await expect(
    page.getByRole("heading", { name: "Compare revisions" }),
  ).toBeVisible();
  await expect(page.getByText("Original history body.")).toBeVisible();
  await expect(page.getByText("Updated history body.")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-003-final-desktop.jpg",
  });

  await page.getByRole("button", { name: "Revert Changes" }).click();
  await expect(page).toHaveURL(/\/wiki\/.+\/_history$/);
  await expect(page.getByText(/Revert wiki page/)).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    `${seeded.repositoryWikiHref}/${encodeURIComponent(fixture.slug)}/_history`,
  );
  await expect(page.getByRole("heading", { name: "History" })).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-003-final-mobile.jpg",
  });
});
