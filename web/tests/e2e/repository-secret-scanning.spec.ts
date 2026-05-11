import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  runPsql,
  screenshotPath,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

const databaseUrl = process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

function sqlLiteral(value: string) {
  return `'${value.replaceAll("'", "''")}'`;
}

function seedSecretScanningAlerts(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  const decodedOwner = decodeURIComponent(owner);
  const decodedRepo = decodeURIComponent(repo);
  const suffix = decodedRepo.replace(/^tree-nav-/, "");

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
      ),
      repo_actor AS (
        SELECT repository_permissions.user_id
        FROM repository_permissions, target_repo
        WHERE repository_permissions.repository_id = target_repo.id
        ORDER BY repository_permissions.created_at ASC
        LIMIT 1
      ),
      main_ref AS (
        SELECT repository_git_refs.target_commit_id AS commit_id
        FROM repository_git_refs, target_repo
        WHERE repository_git_refs.repository_id = target_repo.id
          AND repository_git_refs.name IN ('main', 'refs/heads/main')
        ORDER BY CASE WHEN repository_git_refs.name = 'refs/heads/main' THEN 0 ELSE 1 END
        LIMIT 1
      ),
      settings AS (
        INSERT INTO repository_security_feature_settings (
          repository_id, feature_key, status, summary, alert_count, private_count, config_href
        )
        SELECT target_repo.id,
               'secret_scanning',
               'enabled',
               'Secret scanning alerts are monitored.',
               2,
               0,
               ${sqlLiteral(`${repositoryHref}/settings/security_analysis`)}
        FROM target_repo
        ON CONFLICT (repository_id, feature_key)
        DO UPDATE SET status = 'enabled',
                      summary = EXCLUDED.summary,
                      alert_count = 2,
                      private_count = 0,
                      config_href = EXCLUDED.config_href
      ),
      github_pattern AS (
        INSERT INTO secret_scanning_patterns (
          slug, provider, secret_type, display_name, result_kind, push_protection_enabled
        )
        VALUES (
          ${sqlLiteral(`github-pat-${suffix}`)},
          'GitHub',
          'github_personal_access_token',
          'GitHub personal access token',
          'provider',
          true
        )
        ON CONFLICT (lower(slug)) DO UPDATE
        SET provider = EXCLUDED.provider,
            secret_type = EXCLUDED.secret_type,
            display_name = EXCLUDED.display_name,
            result_kind = EXCLUDED.result_kind,
            push_protection_enabled = EXCLUDED.push_protection_enabled
        RETURNING id
      ),
      generic_pattern AS (
        INSERT INTO secret_scanning_patterns (
          slug, provider, secret_type, display_name, result_kind, push_protection_enabled
        )
        VALUES (
          ${sqlLiteral(`generic-api-key-${suffix}`)},
          'Generic',
          'generic_api_key',
          'Generic API key',
          'generic',
          false
        )
        ON CONFLICT (lower(slug)) DO UPDATE
        SET provider = EXCLUDED.provider,
            secret_type = EXCLUDED.secret_type,
            display_name = EXCLUDED.display_name,
            result_kind = EXCLUDED.result_kind,
            push_protection_enabled = EXCLUDED.push_protection_enabled
        RETURNING id
      ),
      file_one AS (
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        SELECT target_repo.id,
               main_ref.commit_id,
               '.env',
               'TOKEN=ghp_************',
               md5('secret-env-${suffix}'),
               22
        FROM target_repo, main_ref
        ON CONFLICT (repository_id, commit_id, lower(path))
        DO UPDATE SET content = EXCLUDED.content,
                      oid = EXCLUDED.oid,
                      byte_size = EXCLUDED.byte_size
        RETURNING id, repository_id, commit_id
      ),
      file_two AS (
        INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
        SELECT target_repo.id,
               main_ref.commit_id,
               'docs/example.env',
               'API_KEY=sk_************',
               md5('secret-docs-${suffix}'),
               24
        FROM target_repo, main_ref
        ON CONFLICT (repository_id, commit_id, lower(path))
        DO UPDATE SET content = EXCLUDED.content,
                      oid = EXCLUDED.oid,
                      byte_size = EXCLUDED.byte_size
        RETURNING id, repository_id, commit_id
      ),
      alert_one AS (
        INSERT INTO secret_scanning_alerts (
          repository_id, pattern_id, number, state, fingerprint, secret_hash,
          redacted_secret, redacted_context, result_kind, validity_state
        )
        SELECT target_repo.id,
               github_pattern.id,
               1,
               'open',
               ${sqlLiteral(`secret-${suffix}-1`)},
               encode(sha256(${sqlLiteral(`secret-${suffix}-1`)}::bytea), 'hex'),
               'ghp_************',
               'token=ghp_************',
               'provider',
               'active'
        FROM target_repo, github_pattern
        ON CONFLICT (repository_id, fingerprint)
        DO UPDATE SET state = 'open',
                      redacted_secret = EXCLUDED.redacted_secret,
                      redacted_context = EXCLUDED.redacted_context,
                      validity_state = 'active',
                      updated_at = now()
        RETURNING id, repository_id
      ),
      alert_two AS (
        INSERT INTO secret_scanning_alerts (
          repository_id, pattern_id, number, state, fingerprint, secret_hash,
          redacted_secret, redacted_context, result_kind, validity_state
        )
        SELECT target_repo.id,
               generic_pattern.id,
               2,
               'open',
               ${sqlLiteral(`secret-${suffix}-2`)},
               encode(sha256(${sqlLiteral(`secret-${suffix}-2`)}::bytea), 'hex'),
               'sk_************',
               NULL,
               'generic',
               'unknown'
        FROM target_repo, generic_pattern
        ON CONFLICT (repository_id, fingerprint)
        DO UPDATE SET state = 'open',
                      redacted_secret = EXCLUDED.redacted_secret,
                      redacted_context = EXCLUDED.redacted_context,
                      validity_state = 'unknown',
                      updated_at = now()
        RETURNING id, repository_id
      ),
      location_one AS (
        INSERT INTO secret_scanning_alert_locations (
          alert_id, repository_file_id, commit_id, ref_name, branch_name, path, start_line, redacted_snippet
        )
        SELECT alert_one.id,
               file_one.id,
               file_one.commit_id,
               'refs/heads/main',
               'main',
               '.env',
               12,
               'TOKEN=ghp_************'
        FROM alert_one, file_one
        RETURNING alert_id
      ),
      location_two AS (
        INSERT INTO secret_scanning_alert_locations (
          alert_id, repository_file_id, commit_id, ref_name, branch_name, path, start_line, redacted_snippet
        )
        SELECT alert_two.id,
               file_two.id,
               file_two.commit_id,
               'refs/heads/main',
               'main',
               'docs/example.env',
               3,
               'API_KEY=sk_************'
        FROM alert_two, file_two
        RETURNING alert_id
      ),
      validity_one AS (
        INSERT INTO secret_scanning_validity_checks (alert_id, provider, status, message)
        SELECT alert_one.id, 'GitHub', 'active', 'Provider reported the credential is active.'
        FROM alert_one
      ),
      bypass_one AS (
        INSERT INTO push_protection_bypasses (
          repository_id, alert_id, actor_user_id, ref_name, commit_oid, path, reason, status, redacted_snippet
        )
        SELECT alert_one.repository_id,
               alert_one.id,
               repo_actor.user_id,
               'refs/heads/main',
               'abc123',
               '.env',
               'Needed for local example fixture.',
               'pending_review',
               'TOKEN=ghp_************'
        FROM alert_one
        LEFT JOIN repo_actor ON true
      )
      INSERT INTO secret_scanning_alert_assignees (alert_id, user_id)
      SELECT alert_one.id, repo_actor.user_id
      FROM alert_one, repo_actor
      ON CONFLICT DO NOTHING;
      `,
  ]);
}

function disableSecretScanning(repositoryHref: string) {
  if (!databaseUrl) {
    throw new Error("TEST_DATABASE_URL or DATABASE_URL is required");
  }
  const [, owner, repo] = repositoryHref.split("/");
  runPsql(databaseUrl, [
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `
      UPDATE repository_security_feature_settings
      SET status = 'needs_setup',
          summary = 'Secret scanning is not enabled for this repository.'
      FROM repositories
      LEFT JOIN users ON users.id = repositories.owner_user_id
      LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
      WHERE repository_security_feature_settings.repository_id = repositories.id
        AND repository_security_feature_settings.feature_key = 'secret_scanning'
        AND COALESCE(users.username, organizations.slug) = ${sqlLiteral(decodeURIComponent(owner))}
        AND repositories.name = ${sqlLiteral(decodeURIComponent(repo))};
      `,
  ]);
}

test.skip(
  skipWithoutTestDb(),
  "repository Secret scanning smoke needs a database URL",
);
test.setTimeout(90_000);

test("repository Secret scanning alerts support list filters, row links, disabled state, and screenshot evidence", async ({
  page,
  seed,
  signIn,
}, testInfo) => {
  const seeded = await seed({ scenes: ["treeRefs"] });
  const treeRepositoryHref = seeded.hrefs.treeRepository;
  seedSecretScanningAlerts(treeRepositoryHref);
  await signIn(page, seeded, "owner");

  await page.goto(`${treeRepositoryHref}/security/secret-scanning`);
  await expect(
    page.getByRole("heading", { name: "Secret scanning alerts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Secret scanning Credential exposure findings",
    }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByText("GitHub personal access token").first(),
  ).toBeVisible();
  await expect(page.getByText("ghp_************").first()).toBeVisible();
  await expect(page.getByText("super-secret-value")).toHaveCount(0);
  await expect(page.getByRole("link", { name: ".env:12" })).toHaveAttribute(
    "href",
    /\/blob\/refs%2Fheads%2Fmain\/\.env#L12/,
  );
  await expect(
    page.getByRole("heading", {
      name: "Protected pushes and bypass outcomes",
    }),
  ).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-004-phase2-alerts-list"),
  });

  await page.getByRole("button", { name: "Provider: All providers" }).click();
  await page.getByRole("menuitem", { name: /GitHub/ }).click();
  await expect(page).toHaveURL(/provider=GitHub/);
  await expect(
    page.getByText("GitHub personal access token").first(),
  ).toBeVisible();

  await page.getByRole("button", { name: "Select all visible" }).click();
  await expect(
    page.getByRole("button", { name: "Clear visible" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "GitHub personal access token" }),
  ).toHaveAttribute("href", `${treeRepositoryHref}/security/secret-scanning/1`);

  await page.goto(`${treeRepositoryHref}/security/secret-scanning/1`);
  await expect(
    page.getByRole("heading", { name: "GitHub personal access token" }),
  ).toBeVisible();
  await expect(page.getByText("super-secret-value")).toHaveCount(0);
  await expect(page.getByText("ghp_************").first()).toBeVisible();
  await expect(
    page.getByRole("list", { name: "Secret scanning alert timeline" }),
  ).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-004-phase3-alert-detail"),
  });
  await page.getByRole("button", { name: "Resolve alert" }).click();
  await expect(page.getByText("Resolution saved.")).toBeVisible();
  await expect(page.getByText("Resolved").first()).toBeVisible();
  await page.getByRole("button", { name: "Reopen alert" }).click();
  await expect(page.getByText("Reopen saved.")).toBeVisible();
  await page
    .locator(
      'section[aria-label="Alert triage actions"] input[type="checkbox"]',
    )
    .first()
    .uncheck();
  await page.getByRole("button", { name: "Save assignments" }).click();
  await expect(page.getByText("Assignments saved.")).toBeVisible();
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-004-phase4-push-protection"),
  });
  await page.getByLabel("Token validity").selectOption("inactive");
  await page.getByRole("button", { name: "Save validity" }).click();
  await expect(page.getByText("Validity saved.")).toBeVisible();
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-004-final-list"),
  });

  disableSecretScanning(treeRepositoryHref);
  await page.goto(`${treeRepositoryHref}/security/secret-scanning`);
  await expect(
    page.getByRole("link", { name: "Enable secret scanning" }),
  ).toBeVisible();

  seedSecretScanningAlerts(treeRepositoryHref);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${treeRepositoryHref}/security/secret-scanning`);
  await expect(
    page.getByRole("heading", { name: "Secret scanning alerts" }),
  ).toBeVisible();
  await expect(page.locator("body")).toHaveJSProperty("scrollLeft", 0);
  await expectNoHorizontalOverflow(page);
  await expectNoDeadControls(page);
  await page.screenshot({
    fullPage: true,
    path: screenshotPath(testInfo, "code-security-004-final-mobile"),
  });
});
