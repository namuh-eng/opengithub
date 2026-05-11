import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.setTimeout(60_000);

test.skip(
  skipWithoutTestDb(),
  "organization create E2E needs TEST_DATABASE_URL or DATABASE_URL",
);

test("signed-in organization plan picker opens setup and validates slugs", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["empty"] });
  await signIn(page, seeded);

  await page.goto("/organizations/new");
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await expect(
    page.getByRole("heading", { name: "Create a new organization" }),
  ).toBeVisible();
  await expect(page.getByLabel("Free plan")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Team plan unavailable" }),
  ).toBeDisabled();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-001-final-plan-picker"),
  });

  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Tell us about your organization" }),
  ).toBeVisible();

  const uniqueName = `Phase Two Org ${Date.now().toString(36)}`;
  const normalized = uniqueName.toLowerCase().replaceAll(/\s+/g, "-");
  await page.getByLabel("Organization name *").fill(uniqueName);
  await expect(
    page.getByText(`opengithub.namuh.co/${normalized}`),
  ).toBeVisible();
  await expect(page.getByRole("status")).toContainText(
    `${normalized} is available.`,
  );
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-001-final-setup-form"),
  });

  await page.getByLabel("Organization name *").fill("settings");
  await expect(
    page.getByText(/reserved|already taken|not available/i),
  ).toBeVisible();

  await page.getByLabel("Business or institution").check();
  await expect(page.getByLabel("Company name *")).toBeVisible();
  await page.getByLabel("Contact email *").fill("admin@example.com");
  await page
    .getByLabel("I accept the organization terms for this Free plan.")
    .check();
  await expect(
    page.getByRole("button", { name: "Create organization" }),
  ).toBeDisabled();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-001-final-validation-error"),
  });
});

test("organization create setup stays usable on mobile", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["empty"] });
  await signIn(page, seeded);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/organizations/new");
  await expectNoHorizontalOverflow(page);
  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();
  await page.getByLabel("Organization name *").fill("Mobile Org!!");
  await expect(page.getByText("opengithub.namuh.co/mobile-org")).toBeVisible();
  await page.getByLabel("Contact email *").fill("mobile-admin@example.com");
  await page.getByLabel("Business or institution").check();
  await page
    .getByLabel("Company name *")
    .fill(
      "A very long mobile organization company name that should wrap without creating horizontal overflow",
    );
  await page
    .getByLabel("I accept the organization terms for this Free plan.")
    .check();
  await expectNoHorizontalOverflow(page);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-001-final-mobile"),
  });
});

test("organization create flow supports keyboard-only plan selection and error recovery", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed({ scenes: ["empty"] });
  await signIn(page, seeded);

  await page.goto("/organizations/new");
  const freeButton = page.getByRole("button", {
    name: "Create a free organization",
  });
  await freeButton.focus();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("heading", { name: "Tell us about your organization" }),
  ).toBeFocused();

  await page.getByLabel("Organization name *").fill("settings");
  await expect(
    page.getByText(/reserved|already taken|not available/i),
  ).toBeVisible();
  await page
    .getByLabel("Organization name *")
    .fill(`Recovered Keyboard Org ${Date.now().toString(36)}`);
  await expect(page.getByText(/is available/i)).toBeVisible();
  await page.getByLabel("Contact email *").fill("keyboard@example.com");
  await page
    .getByLabel("I accept the organization terms for this Free plan.")
    .check();
  await expect(
    page.getByRole("button", { name: "Create organization" }),
  ).toBeEnabled();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
});

test("signed-in user creates a free organization and sees it in navigation", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["empty"] });
  await signIn(page, seeded);

  await page.goto("/organizations/new");
  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();

  const uniqueName = `Created Org ${Date.now().toString(36)}`;
  const normalized = uniqueName.toLowerCase().replaceAll(/\s+/g, "-");
  await page.getByLabel("Organization name *").fill(uniqueName);
  await expect(page.getByText(`${normalized} is available.`)).toBeVisible();
  await page.getByLabel("Contact email *").fill("admin@example.com");
  await page.getByLabel("Business or institution").check();
  await page.getByLabel("Company name *").fill("Created Org Inc.");
  await page
    .getByLabel("I accept the organization terms for this Free plan.")
    .check();
  await page.getByRole("button", { name: "Create organization" }).click();

  await expect(page).toHaveURL(new RegExp(`/orgs/${normalized}(?:$|[/?#])`));
  await expect(page.getByRole("heading", { name: uniqueName })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.reload();
  await page.getByRole("button", { name: "Global menu" }).click();
  await expect(page.getByRole("menuitem", { name: uniqueName })).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-001-final-create-redirect"),
  });

  await page.goto("/organizations/new");
  await page
    .getByRole("button", { name: "Create a free organization" })
    .click();
  await page.getByLabel("Organization name *").fill(uniqueName);
  await expect(
    page.getByText(/already taken|not available|reserved/i),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Create organization" }),
  ).toBeDisabled();
});
