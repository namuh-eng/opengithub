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
    latestRevisionId?: string;
    revision?: { id: string };
  };
  redirectHref: string;
};

type WikiEditView = {
  page: {
    title: string;
    slug: string;
    markdown: string;
    latestRevisionId: string;
  };
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
    "Rust API did not become healthy for repository wiki editing E2E",
  );
}

function repositoryParts(repositoryWikiHref: string) {
  const [, owner, repo] = repositoryWikiHref.split("/");
  return { owner, repo };
}

test.skip(
  !databaseUrl,
  "Repository Wiki editing E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.beforeEach(async ({ page }) => {
  await waitForApiHealth(page);
});

test("signed-in repository wiki supports pages index, create, preview, save, edit, conflict handling, and mobile layout", async ({
  page,
}) => {
  const seeded = seedWiki();
  await signIn(page, seeded);
  const cookie = `${seeded.cookieName}=${seeded.cookieValue}`;
  const { owner, repo } = repositoryParts(seeded.repositoryWikiHref);
  const unique = Date.now().toString(36);
  const title = `Release Notes ${unique}`;

  await page.goto(`${seeded.repositoryWikiHref}/_pages`);
  await expect(page.getByRole("heading", { name: "Pages" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "New Page" }).first(),
  ).toHaveAttribute("href", `/${owner}/${repo}/wiki/_new`);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  await page.getByRole("link", { name: "New Page" }).first().click();
  await expect(page.getByRole("heading", { name: "New Page" })).toBeVisible();
  await page.getByLabel("Page title").fill(title);
  await page
    .getByLabel("Wiki page source")
    .fill(`# ${title}\n\nInitial release notes.`);
  await page.getByLabel("Edit message").fill("   ");
  await page.getByRole("button", { name: "Save Page" }).click();
  await expect(page.locator('.card [role="alert"]')).toContainText(
    "wiki edit message is required",
  );
  await expect(page).toHaveURL(/\/wiki\/_new$/);
  await page.getByLabel("Page title").fill(".");
  await page.getByLabel("Edit message").fill(`Create invalid ${unique}`);
  await page.getByRole("button", { name: "Save Page" }).click();
  await expect(page.locator('.card [role="alert"]')).toContainText(
    "wiki page slug is invalid",
  );
  await page.getByLabel("Page title").fill(title);
  await page.getByLabel("Image URL").fill("https://images.example/release.png");
  await page.getByLabel("Alt text").fill("Release diagram");
  await page.getByRole("button", { name: "Insert image" }).click();
  await expect(page.getByRole("status")).toContainText(
    "Image reference inserted.",
  );
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByRole("status")).toContainText("Preview rendered.");
  await expect(page.getByRole("heading", { name: title })).toBeVisible();
  await page.getByLabel("Edit message").fill(`Create ${title}`);
  await page.getByRole("button", { name: "Save Page" }).click();
  await expect(page).toHaveURL(
    new RegExp(`/wiki/${encodeURIComponent(title)}$`),
  );
  await expect(page.locator("#repository-wiki-title")).toHaveText(title);
  await expect(page.getByAltText("Release diagram")).toBeVisible();

  await page.goto(
    `${seeded.repositoryWikiHref}/${encodeURIComponent(title)}/_edit`,
  );
  await expect(
    page.getByRole("heading", { name: `Edit ${title}` }),
  ).toBeVisible();
  await page
    .getByLabel("Wiki page source")
    .fill(`# ${title}\n\nUpdated from the signed-in E2E sweep.`);
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByRole("status")).toContainText("Preview rendered.");
  await page.getByLabel("Edit message").fill(`Update ${title}`);
  await page.getByRole("button", { name: "Save Page" }).click();
  await expect(
    page.getByText("Updated from the signed-in E2E sweep."),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-002-final-desktop.jpg",
  });

  const editResponse = await page.request.get(
    `http://localhost:3016/api/repos/${owner}/${repo}/wiki/${encodeURIComponent(
      title,
    )}/edit`,
    { headers: { cookie } },
  );
  expect(editResponse.status()).toBe(200);
  const editView = (await editResponse.json()) as WikiEditView;
  const staleRevisionId = editView.page.latestRevisionId;
  const firstPatch = await page.request.patch(
    `http://localhost:3016/api/repos/${owner}/${repo}/wiki/${encodeURIComponent(
      title,
    )}`,
    {
      headers: { cookie },
      data: {
        title,
        markdown: `# ${title}\n\nConflict setup.`,
        message: `Conflict setup ${unique}`,
        editMode: "markdown",
        expectedRevisionId: staleRevisionId,
      },
    },
  );
  expect(firstPatch.status()).toBe(200);
  const firstPatchBody = (await firstPatch.json()) as WikiMutationResult;
  expect(firstPatchBody.redirectHref).toContain(encodeURIComponent(title));
  const stalePatch = await page.request.patch(
    `http://localhost:3016/api/repos/${owner}/${repo}/wiki/${encodeURIComponent(
      title,
    )}`,
    {
      headers: { cookie },
      data: {
        title,
        markdown: `# ${title}\n\nStale body.`,
        message: `Stale conflict ${unique}`,
        editMode: "markdown",
        expectedRevisionId: staleRevisionId,
      },
    },
  );
  expect(stalePatch.status()).toBe(409);

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.repositoryWikiHref}/${encodeURIComponent(title)}`);
  await expect(page.locator("#repository-wiki-title")).toHaveText(title);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: "../ralph/screenshots/build/wiki-002-final-mobile.jpg",
  });
});
