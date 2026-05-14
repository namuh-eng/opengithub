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
  "file finder E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test.setTimeout(120_000);

test("repository t shortcut opens full-ref file finder with local fuzzy navigation", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  await signIn(page, seeded);
  const repositoryHref = seeded.hrefs.treeRepository;
  const repositoryName = repositoryHref.split("/").at(-1);
  if (!repositoryName) {
    throw new Error("tree repository href did not include a repo name");
  }

  await page.goto(`${repositoryHref}/tree/feature%2Ftree-nav`);
  await expect(
    page.getByLabel("Switch branches or tags. Current ref feature/tree-nav"),
  ).toBeVisible();

  await page.keyboard.press("t");
  await expect(page).toHaveURL(
    new RegExp(`/${repositoryName}/find/feature%2Ftree-nav$`),
  );

  const input = page.getByRole("combobox", {
    name: "Fuzzy-find a file path",
  });
  await expect(input).toBeFocused();
  await expect(page.getByText("75 cached paths")).toBeVisible();
  await expect(page.getByRole("option", { name: /README\.md/ })).toBeVisible();
  await expect(
    page.getByRole("option", { name: /docs\/example-071\.md/ }),
  ).toBeVisible();

  await input.fill("zzzz-no-match");
  await expect(page.getByRole("status")).toContainText("No matching files");
  await input.press("Escape");
  await expect(input).toHaveValue("");
  await expect(page.getByText("75 cached paths")).toBeVisible();

  await input.fill("example");
  await expect(page.getByText(/72 matching paths/)).toBeVisible();
  await expect(
    page.getByRole("option", { name: /docs\/example-000\.md/ }),
  ).toHaveAttribute("aria-selected", "true");
  await input.press("ArrowDown");
  await expect(
    page.getByRole("option", { name: /docs\/example-001\.md/ }),
  ).toHaveAttribute("aria-selected", "true");
  await input.press("Enter");
  await expect(page).toHaveURL(
    new RegExp(
      `/${repositoryName}/blob/feature%2Ftree-nav/docs/example-001.md$`,
    ),
  );
  await expect(
    page.getByRole("heading", { name: "docs/example-001.md" }),
  ).toBeVisible();

  await page.goto(`${repositoryHref}/find/feature%2Ftree-nav`);
  await input.fill("guide");
  await expect(
    page.getByRole("option", { name: /docs\/guide\.md/ }),
  ).toHaveAttribute(
    "href",
    new RegExp(`/${repositoryName}/blob/feature%2Ftree-nav/docs/guide.md$`),
  );
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "search-007-file-finder-final"),
  });
});
