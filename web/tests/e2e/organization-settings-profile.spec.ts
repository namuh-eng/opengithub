import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

function slugFromProfileHref(href: string) {
  const parts = href.split("/").filter(Boolean);
  const organizationsIndex = parts.indexOf("organizations");
  if (organizationsIndex >= 0 && parts[organizationsIndex + 1]) {
    return parts[organizationsIndex + 1];
  }
  const orgsIndex = parts.indexOf("orgs");
  if (orgsIndex >= 0 && parts[orgsIndex + 1]) {
    return parts[orgsIndex + 1];
  }
  throw new Error(
    `seeded organization profile href did not include a slug: ${href}`,
  );
}

test.skip(
  skipWithoutTestDb(),
  "organization settings profile E2E needs a test database",
);
test.setTimeout(60_000);

test("owner opens organization profile settings shell", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["orgProfile"] });
  await signIn(page, seeded, "owner");
  const slug = slugFromProfileHref(seeded.hrefs.organizationProfile);

  await page.goto(`/organizations/${slug}/settings/profile`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Profile" }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", {
      name: "Organization settings navigation",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Personal settings" }),
  ).toHaveAttribute("href", "/settings/profile");
  await expect(
    page.getByRole("link", { name: "Organization settings" }),
  ).toHaveAttribute("href", `/organizations/${slug}/settings/profile`);
  await expect(
    page.getByRole("link", { name: "View organization" }),
  ).toHaveAttribute("href", `/orgs/${slug}`);
  await expect(page.getByLabel("Organization display name")).toBeVisible();
  await expect(page.getByLabel("Contact email")).toBeVisible();
  await expect(
    page.getByRole("textbox", { exact: true, name: "X" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Upload unavailable" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Save profile changes" }),
  ).toBeDisabled();
  await expect(page.getByText("Billing", { exact: true })).toHaveAttribute(
    "aria-disabled",
    "true",
  );
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-phase2-settings-shell"),
  });
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-final-desktop-shell"),
  });

  await page.goto(`/orgs/${slug}/settings`);
  await expect(page).toHaveURL(`/organizations/${slug}/settings/profile`);
});

test("owner saves organization profile, contact, and social sections", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["orgProfile"] });
  await signIn(page, seeded, "owner");
  const slug = slugFromProfileHref(seeded.hrefs.organizationProfile);

  await page.goto(`/organizations/${slug}/settings/profile`);
  await page
    .getByLabel("Organization display name")
    .fill(`Editorial ${slug.slice(-6)}`);
  await page
    .getByLabel("Description")
    .fill("A persisted organization profile update from the browser.");
  await page.getByLabel("URL").fill("https://opengithub.namuh.co");
  await page.getByLabel("Location").fill("Seoul, KR");
  await page.getByLabel("Public email").fill("public@example.com");
  await page.getByRole("button", { name: "Save profile changes" }).click();
  await expect(page.getByText("Public profile updated")).toBeVisible();

  await page.getByLabel("Contact email").fill("owners@example.com");
  await page.getByLabel("Billing email").fill("finance@example.com");
  await page.getByRole("button", { name: "Save contact changes" }).click();
  await expect(page.getByText("Administrative contact updated")).toBeVisible();

  await page
    .getByRole("textbox", { exact: true, name: "X" })
    .fill("@opengithub");
  await page
    .getByRole("textbox", { exact: true, name: "Mastodon" })
    .fill("https://social.example/@opengithub");
  await page.getByRole("button", { name: "Save social accounts" }).click();
  await expect(page.getByText("Social accounts updated")).toBeVisible();
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-phase3-profile-save"),
  });
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-final-profile-form"),
  });

  await page.reload();
  await expect(page.getByLabel("Public email")).toHaveValue(
    "public@example.com",
  );
  await expect(page.getByLabel("Contact email")).toHaveValue(
    "owners@example.com",
  );
  await expect(
    page.getByRole("textbox", { exact: true, name: "X" }),
  ).toHaveValue("@opengithub");

  await page.getByLabel("URL").fill("javascript:alert(1)");
  await page.getByRole("button", { name: "Save profile changes" }).click();
  await expect(
    page.getByText("URL must start with http:// or https://."),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-final-validation-error"),
  });
});

test("owner validates rename and typed danger guardrails", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["orgProfile"] });
  await signIn(page, seeded, "owner");
  const slug = slugFromProfileHref(seeded.hrefs.organizationProfile);
  const nextSlug = `${slug}-renamed`;

  await page.goto(`/organizations/${slug}/settings/profile`);
  await page.getByLabel("New organization slug").fill("admin");
  await page.getByLabel("Confirm current slug").fill(slug);
  await page.getByRole("button", { name: "Rename" }).click();
  await expect(
    page.getByText("This organization slug is not available."),
  ).toBeVisible();

  await page.getByLabel("New organization slug").fill(nextSlug);
  await page.getByLabel("Confirm current slug").fill(slug);
  await page.getByRole("button", { name: "Rename" }).click();
  await expect(page.getByText("Organization renamed")).toBeVisible();
  await expect(page).toHaveURL(`/organizations/${nextSlug}/settings/profile`);
  await expect(page.getByLabel("New organization slug")).toHaveValue(nextSlug);

  await page.getByRole("button", { name: "Archive organization" }).click();
  await expect(
    page.getByRole("dialog", { name: "Archive organization" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Type slug to archive" }),
  ).toBeDisabled();
  await page.getByLabel(`Confirm archive ${nextSlug}`).fill(nextSlug);
  await expect(
    page.getByRole("button", { name: "Archive unavailable" }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(
    page.getByRole("dialog", { name: "Archive organization" }),
  ).toHaveCount(0);

  await page.getByRole("button", { name: "Delete organization" }).click();
  await page.getByLabel(`Confirm delete ${nextSlug}`).fill(nextSlug);
  await expect(
    page.getByRole("button", { name: "Delete unavailable" }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expectNoDeadControls(page);

  await page.setViewportSize({ width: 390, height: 900 });
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-phase4-danger-zone"),
  });
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-final-danger-zone"),
  });
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "org-admin-002-final-mobile"),
  });
});
