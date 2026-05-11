import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  requireTestDatabase,
  runPsql,
  type SeedResult,
  type SeedSpec,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedSecurityOverview(repositoryHref: string) {
  const databaseUrl = requireTestDatabase();
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");
  const policyMarkdown =
    "# Security policy\n\nPlease email [security](mailto:security@example.com).\n\n<script>alert('x')</script>";
  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
      WITH target_repo AS (
        SELECT repositories.id, repositories.default_branch
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      ),
      target_ref AS (
        SELECT repository_git_refs.target_commit_id AS commit_id
        FROM repository_git_refs
        JOIN target_repo ON target_repo.id = repository_git_refs.repository_id
        WHERE repository_git_refs.name = 'refs/heads/main'
        LIMIT 1
      ),
      policy_file AS (
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        SELECT target_repo.id,
               target_ref.commit_id,
               'SECURITY.md',
               ${sqlLiteral(policyMarkdown)},
               'security-policy-${suffix}',
               length(${sqlLiteral(policyMarkdown)})
        FROM target_repo, target_ref
        ON CONFLICT (repository_id, commit_id, lower(path))
        DO UPDATE SET content = EXCLUDED.content, oid = EXCLUDED.oid, byte_size = EXCLUDED.byte_size
        RETURNING repository_id
      )
      INSERT INTO repository_security_feature_settings (
        repository_id, feature_key, status, summary, alert_count, private_count, config_href
      )
      SELECT target_repo.id, feature_key, status, summary, alert_count, private_count, config_href
      FROM target_repo,
      (VALUES
        ('dependabot', 'enabled', 'Dependency alerts are monitored.', 7::bigint, 2::bigint, ${sqlLiteral(`${repositoryHref}/security/dependabot`)}),
        ('code_scanning', 'needs_setup', 'No code scanning workflow is configured.', 3::bigint, 1::bigint, ${sqlLiteral(`${repositoryHref}/security/code-scanning/setup`)}),
        ('secret_scanning', 'disabled', 'Secret scanning is not enabled.', 0::bigint, 0::bigint, NULL)
      ) AS feature_rows(feature_key, status, summary, alert_count, private_count, config_href)
      ON CONFLICT (repository_id, feature_key)
      DO UPDATE SET status = EXCLUDED.status,
                    summary = EXCLUDED.summary,
                    alert_count = EXCLUDED.alert_count,
                    private_count = EXCLUDED.private_count,
                    config_href = EXCLUDED.config_href,
                    updated_at = now();

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      INSERT INTO repository_security_advisories (
        repository_id, advisory_identifier, severity, status, title, summary,
        package_name, vulnerable_range, advisory_href, published_at
      )
      SELECT target_repo.id,
             'GHSA-visible-${suffix}',
             'high',
             'published',
             'Visible advisory',
             'Patch the affected dependency.',
             'demo-package',
             '< 1.2.3',
             ${sqlLiteral(`${repositoryHref}/security/advisories/GHSA-visible-${suffix}`)},
             now() - interval '1 hour'
      FROM target_repo
      ON CONFLICT (repository_id, advisory_identifier)
      DO UPDATE SET title = EXCLUDED.title,
                    summary = EXCLUDED.summary,
                    status = 'published',
                    updated_at = now();
      `,
  ]);
}

function deleteSecurityPolicy(repositoryHref: string) {
  const databaseUrl = requireTestDatabase();
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      DELETE FROM repository_security_policies
      USING target_repo
      WHERE repository_security_policies.repository_id = target_repo.id;

      WITH target_repo AS (
        SELECT repositories.id
        FROM repositories
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodedOwner)}
          AND repositories.name = ${sqlLiteral(decodedRepo)}
        LIMIT 1
      )
      DELETE FROM repository_files
      USING target_repo
      WHERE repository_files.repository_id = target_repo.id
        AND lower(repository_files.path) IN (
          'security.md',
          '.github/security.md',
          'docs/security.md'
        );
      `,
  ]);
}

async function seedSecurityRepository(
  seed: (spec?: SeedSpec) => Promise<SeedResult>,
) {
  const seeded = await seed({ scenes: ["treeRefs"] });
  seedSecurityOverview(seeded.hrefs.treeRepository);
  return seeded;
}

test.skip(
  skipWithoutTestDb(),
  "repository Security smoke needs a database URL",
);
test.setTimeout(150_000);

test("repository Security overview renders policy, feature cards, and advisory links", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seedSecurityRepository(seed);
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.treeRepository}/security`);
  await expect(
    page.getByRole("heading", { name: "Security overview" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Overview Policy, feature state, and advisories",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { exact: true, name: "Security" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.getByText("Private counts visible")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "SECURITY.md" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Security policy" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "security" }),
  ).toHaveAttribute("href", "mailto:security@example.com");
  await expect(page.locator("script", { hasText: "alert" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Source" })).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/blob/main/SECURITY.md`,
  );
  await expect(
    page.getByRole("link", { name: "Dependabot" }).last(),
  ).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/security/dependabot`,
  );
  await expect(
    page.getByText("Dependency alerts are monitored."),
  ).toBeVisible();
  await expect(page.getByText("7", { exact: true })).toBeVisible();
  await expect(page.getByText("Visible advisory")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Visible advisory" }),
  ).toHaveAttribute("href", /\/security\/advisories\/GHSA-visible-/);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-001-phase2-overview"),
  });
});

test("repository Security policy renders markdown anchors and file actions", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seedSecurityRepository(seed);
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.treeRepository}/security/policy`);
  await expect(
    page.getByRole("heading", { exact: true, name: "Security policy" }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Security policy Responsible disclosure guidance",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("heading", { name: "SECURITY.md" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "security" }),
  ).toHaveAttribute("href", "mailto:security@example.com");
  await expect(
    page
      .getByRole("navigation", { name: "Policy headings" })
      .getByRole("link", {
        name: "Security policy",
      }),
  ).toHaveAttribute("href", "#security-policy");
  await expect(
    page.getByRole("link", { exact: true, name: "Source" }),
  ).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/blob/main/SECURITY.md`,
  );
  await expect(
    page.getByRole("link", { exact: true, name: "Raw" }),
  ).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/raw/main/SECURITY.md`,
  );
  await expect(
    page.getByRole("link", { exact: true, name: "History" }),
  ).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/commits/main/SECURITY.md`,
  );
  await expect(
    page.getByRole("link", { name: "Initial commit" }),
  ).toHaveAttribute("href", /\/commit\/[a-f0-9]+/);
  await expect(page.locator("script", { hasText: "alert" })).toHaveCount(0);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-001-phase3-policy"),
  });
});

test("repository Security policy editor commits changes to file and raw views", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seedSecurityRepository(seed);
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.treeRepository}/security/policy/edit`);
  await expect(
    page.getByRole("heading", { name: "Edit security policy" }),
  ).toBeVisible();
  await page
    .getByLabel("Markdown")
    .fill(
      "# Security policy\n\nEmail [triage](mailto:triage@example.com).\n\n## Scope\n\nDefault branch only.",
    );
  await page.getByRole("tab", { name: "Preview" }).click();
  await expect(page.getByText("Default branch only.")).toBeVisible();
  await page.getByLabel("Commit message").fill("Update security policy");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(
    page.getByText("Security policy saved to the default branch."),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "View file" })).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/blob/main/SECURITY.md`,
  );

  await page.getByRole("link", { name: "Open raw" }).click();
  await expect(page.getByText("triage@example.com")).toBeVisible();

  await page.goto(`${seeded.hrefs.treeRepository}/security/policy`);
  await expect(
    page.getByRole("link", { exact: true, name: "triage" }),
  ).toHaveAttribute("href", "mailto:triage@example.com");
  await expect(
    page.getByRole("link", { name: "Update security policy" }),
  ).toHaveAttribute("href", /\/commit\/[a-f0-9]+/);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-001-phase4-policy-edit"),
  });
});

test("repository Security final smoke covers missing policy and mobile layout", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seedSecurityRepository(seed);
  await signIn(page, seeded);

  await page.goto(`${seeded.hrefs.treeRepository}/security`);
  await expect(
    page.getByRole("heading", { name: "Security overview" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { exact: true, name: "Security policy" }),
  ).toHaveAttribute("href", `${seeded.hrefs.treeRepository}/security/policy`);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-001-final-overview"),
  });

  await page.goto(`${seeded.hrefs.treeRepository}/security/policy/edit`);
  await page
    .getByLabel("Markdown")
    .fill(
      "# Security policy\n\nEmail [mobile triage](mailto:mobile@example.com).\n\n## Very long disclosure section\n\nThe security policy body wraps on constrained screens without overlapping the sidebar, editor actions, or repository header.",
    );
  await page.getByLabel("Commit message").fill("Finalize security policy");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(
    page.getByText("Security policy saved to the default branch."),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${seeded.hrefs.treeRepository}/security/policy`);
  await expect(
    page.getByRole("link", { exact: true, name: "mobile triage" }),
  ).toHaveAttribute("href", "mailto:mobile@example.com");
  await expect(
    page
      .getByRole("complementary", { name: "Security and quality navigation" })
      .getByRole("link", {
        name: "Security policy Responsible disclosure guidance",
      }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-001-final-mobile"),
  });

  deleteSecurityPolicy(seeded.hrefs.treeRepository);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(`${seeded.hrefs.treeRepository}/security/policy`);
  await expect(
    page.getByRole("heading", { name: "No published policy" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Start setup" })).toHaveAttribute(
    "href",
    `${seeded.hrefs.treeRepository}/security/policy/edit`,
  );
  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
});
