import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(
  skipWithoutTestDb(),
  "repository Dependency graph smoke needs a database URL",
);
test.setTimeout(90_000);

test("repository Dependencies renders filters, rows, and concrete actions", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["treeRefs", "dependencyGraph"] });
  await signIn(page, seeded);
  const repositoryHref = seeded.hrefs.treeRepository;

  await page.goto(`${repositoryHref}/network/dependencies`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Dependencies" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Dependency graph Dependencies and dependents",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { exact: true, name: "Dependencies" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { exact: true, name: "Dependents" }),
  ).toHaveAttribute("href", `${repositoryHref}/network/dependents`);
  await expect(page.getByLabel("Dependency summary metrics")).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Repository dependencies list" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Search" }).fill("playwright");
  await expect(page.getByRole("button", { name: "Apply" })).toBeVisible();
  await page.goto(`${repositoryHref}/network/dependencies?q=playwright`);
  await expect(page).toHaveURL(/q=playwright/);
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();

  const npmFilterHref = await page
    .getByRole("region", { name: "Dependency ecosystem totals" })
    .getByRole("link", { name: /npm\s+\d+/ })
    .getAttribute("href");
  expect(npmFilterHref).toMatch(/ecosystem=npm/);
  await page.goto(npmFilterHref ?? "");
  await expect(page).toHaveURL(/ecosystem=npm/);
  await expect(
    page.getByRole("link", { exact: true, name: "@playwright/test" }),
  ).toBeVisible();

  const directFilterHref = await page
    .getByRole("link", { name: "Direct" })
    .getAttribute("href");
  expect(directFilterHref).toMatch(/relationship=direct/);
  await page.goto(`${repositoryHref}/network/dependencies?relationship=direct`);
  await expect(page).toHaveURL(/relationship=direct/);
  await expect(
    page.getByRole("list", { name: "Indexed dependency manifests" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Search" }).fill("not-present");
  await expect(page.getByRole("button", { name: "Apply" })).toBeVisible();
  await page.goto(`${repositoryHref}/network/dependencies?q=not-present`);
  await expect(
    page.getByRole("heading", {
      name: "No matching dependencies were found.",
    }),
  ).toBeVisible();
  const exportResponse = await page.request.post(
    `${repositoryHref}/network/dependencies/sbom`,
  );
  expect(exportResponse.status()).toBe(201);
  const exportJob = await exportResponse.json();
  expect(exportJob.status).toBe("ready");
  expect(exportJob.downloadHref).toMatch(/\/network\/dependencies\/sbom\/.+/);
  await page.goto(`${repositoryHref}/network/dependencies`);
  await expect(page.getByText("Latest SBOM ready")).toBeVisible();
  const sbom = await page.request.get(exportJob.downloadHref);
  expect(sbom.status()).toBe(200);
  expect(sbom.headers()["content-type"]).toContain("json");
  expect(sbom.headers()["content-disposition"]).toContain("attachment");
  const sbomBody = await sbom.json();
  expect(sbomBody.spdxVersion).toBe("SPDX-2.3");
  expect(JSON.stringify(sbomBody)).toContain("@playwright/test");
  await expectNoDeadControls(page);

  const dependentSuffix = decodeURIComponent(
    repositoryHref.split("/")[2],
  ).replace(/^tree-nav-/, "");
  const dependentOwner = `public-consumer-${dependentSuffix}`;
  await page.goto(`${repositoryHref}/network/dependents`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Dependents" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "Dependents" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.getByLabel("Dependents summary metrics")).toBeVisible();
  await expect(page.getByText("Counts are approximate")).toBeVisible();
  await page.getByText("Counts are approximate").click();
  await expect(page.getByText(/Private consumers are counted/)).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Repository dependents list" }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("link", { name: /public-consumer-.+\/workflow-tools-/ })
      .first(),
  ).toBeVisible();
  await expect(page.getByText(/private-workflow-tools/)).toHaveCount(0);

  await page.getByRole("button", { name: "Package: All packages" }).click();
  await page.getByRole("menuitem", { name: /npm:@playwright\/test/ }).click();
  await expect(page).toHaveURL(/package=npm%3A%40playwright%2Ftest/);
  await page.getByRole("textbox", { name: "Owner" }).fill(dependentOwner);
  await page.getByRole("button", { name: "Apply owner" }).click();
  await expect(page).toHaveURL(new RegExp(`owner=${dependentOwner}`));
  await expect(
    page.getByRole("list", { name: "Repository dependents list" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Owner" }).fill("missing-owner");
  await page.getByRole("button", { name: "Apply owner" }).click();
  await expect(
    page.getByRole("heading", {
      name: "No public dependents matched these filters.",
    }),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { exact: true, name: "Dependents" }),
  ).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth,
  );
  expect(horizontalOverflow).toBe(false);
  await expectNoHorizontalOverflow(page);
  await expectNoDeadControls(page);
});
