import type { Page } from "@playwright/test";
import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(
  skipWithoutTestDb(),
  "owner packages E2E needs TEST_DATABASE_URL or DATABASE_URL",
);
test.setTimeout(120000);

function ownerFromRepositoryHref(href: string) {
  const [owner] = href.split("/").filter(Boolean);
  if (!owner) {
    throw new Error(`Unexpected repository href: ${href}`);
  }
  return owner;
}

function orgFromHref(href: string) {
  const [, org] = href.split("/").filter(Boolean);
  if (!org) {
    throw new Error(`Unexpected organization href: ${href}`);
  }
  return org;
}

async function waitForApi(page: Page) {
  await expect
    .poll(
      async () => {
        try {
          const response = await page.request.get(
            "http://localhost:3016/health",
            {
              timeout: 1000,
            },
          );
          return response.status() < 500;
        } catch {
          return false;
        }
      },
      { timeout: 60000 },
    )
    .toBe(true);
}

test("owner package lists enforce visibility and preserve package filters", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  await waitForApi(page);
  const seeded = await seed({ scenes: ["orgProfile", "ownerPackages"] });
  const owner = ownerFromRepositoryHref(seeded.hrefs.firstRepository);
  const org = orgFromHref(seeded.hrefs.organizationProfile);

  await page.goto(`/${owner}?tab=packages`);
  await expect(
    page.getByRole("link", { name: /^list-container-/ }),
  ).toBeVisible();
  await expect(page.getByText(/^list-private-/)).toHaveCount(0);
  await expect(
    page.getByRole("link", { name: "Linked artifacts" }),
  ).toHaveAttribute(
    "href",
    new RegExp(`^/${owner}\\?tab=packages&artifactTab=artifacts$`),
  );
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  await signIn(page, seeded, "owner");
  await page.goto(`/${owner}?tab=packages&type=npm&visibility=private`);
  await expect(
    page.getByRole("link", { name: /^list-private-/ }),
  ).toBeVisible();
  await expect(page.getByLabel("Type")).toHaveValue("npm");
  await expect(page.getByLabel("Visibility")).toHaveValue("private");

  await page.context().clearCookies();
  await page.goto(`/orgs/${org}/packages`);
  await expect(page.getByRole("link", { name: /^org-public-/ })).toBeVisible();
  await expect(page.getByText(/^org-internal-/)).toHaveCount(0);
  await expect(page.getByText(/^org-private-/)).toHaveCount(0);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);

  await signIn(page, seeded, "collaborator");
  await page.goto(`/orgs/${org}/packages?type=nuget&visibility=internal`);
  await expect(
    page.getByRole("link", { name: /^org-internal-/ }),
  ).toBeVisible();
  await expect(page.getByText(/^org-private-/)).toHaveCount(0);
  await expect(page.getByLabel("Type")).toHaveValue("nuget");
  await expect(page.getByLabel("Visibility")).toHaveValue("internal");

  await page.goto(`/orgs/${org}/packages?artifactTab=artifacts`);
  await expect(
    page.getByRole("heading", { name: "Linked artifacts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "View GitHub Packages" }),
  ).toHaveAttribute("href", `/orgs/${org}/packages`);
  await page.screenshot({
    path: screenshotPath(testInfo, "packages-001-owner-lists"),
  });
});
