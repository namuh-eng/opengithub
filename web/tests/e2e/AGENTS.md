# E2E test contract — opengithub

This directory holds Playwright specs. **All new signed-in specs MUST go through `_fixtures/auth.ts`.** Do not call the Rust seeder yourself, do not mint cookies inline, do not duplicate `seedSession` / `seedDashboard`.

## Local prerequisites

```sh
make db-up-test          # one-time: bring up the isolated test Postgres on :55433
make test-e2e            # runs the suite — auto-loads .env.test
make db-down-test        # tear down (drops the volume; next run re-migrates)
```

If `make test-e2e` aborts with `TEST_DATABASE_URL or DATABASE_URL is required`, you forgot `make db-up-test` (or you sourced a different `.env`). The fixture refuses to silently fall back to anonymous state.

## Writing a new signed-in spec

```ts
import {
  expect,
  expectNoDeadControls,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(skipWithoutTestDb(), "needs TEST_DATABASE_URL or DATABASE_URL");

test("dashboard renders the seeded primary repo", async ({ page, seed, signIn }) => {
  const seeded = await seed();           // owner + collaborator cookies
  await signIn(page, seeded, "owner");   // default persona is "owner"

  await page.goto(seeded.hrefs.firstRepository);
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expectNoDeadControls(page);
});
```

That's the whole template. No `execFileSync`, no `addCookies`, no env-var magic.

## Personas

| persona        | who they are                                | available when               |
| -------------- | ------------------------------------------- | ---------------------------- |
| `owner`        | "Dashboard Tester", owns the seeded repos   | always                       |
| `collaborator` | "Profile Action Viewer", repo collaborator  | always                       |
| `outsider`     | read-only viewer, distinct user account     | when `scenes: ["treeRefs"]`  |

Switching personas mid-test:

```ts
await signIn(page, seeded, "collaborator");
await page.goto(seeded.hrefs.firstRepository);
// later:
await page.context().clearCookies();
await signIn(page, seeded, "owner");
```

To request a 4th persona, extend `Persona` in `_fixtures/auth.ts` and the corresponding cookie field in `crates/api/examples/dashboard_e2e_seed.rs` (Solution 3 of the E2E standardization brief replaces this with a `personas` array — until then, both files have to change together).

## Scenes

`seed({ scenes: [...] })` opts into seeded data beyond the default dashboard:

| scene name         | what it adds                                       |
| ------------------ | -------------------------------------------------- |
| `empty`            | empty dashboard (no repos, no notifications)       |
| `treeRefs`         | tree-view repo + the `outsider` cookie             |
| `forkRefs`         | fork chain + compare data                          |
| `blobEdge`         | blob view edge cases (binary, LFS, large)          |
| `pullRequestMerge` | merge-ready PR + commits/diff                      |
| `actionsRunDetail` | Actions run + jobs + steps + logs                  |
| `issueTemplate`    | issue templates / forms                            |
| `orgProfile`       | organization profile + empty teams                 |
| `accountSecurity`  | session list / 2FA fixtures                        |

Combine freely: `seed({ scenes: ["pullRequestMerge", "actionsRunDetail"] })`.

Available hrefs are on `seeded.hrefs.*`. If the scene that mints a given href isn't requested, the field is `""` — assert before using.

## Non-negotiables (from web/CLAUDE.md)

- Editorial design system only — no GitHub blue/green/red, no Primer, no Octicons.
- Every spec that asserts a page should call `expectNoDeadControls(page)` and `expectNoHorizontalOverflow(page)` on at least the primary page.
- Screenshots saved for review go to `ralph/screenshots/build/<feature>.jpg` — use `screenshotPath(testInfo, "<feature>")`.

## What NOT to do

- **Do not** import `execFileSync` in a spec. The fixture is the only file that runs the seeder.
- **Do not** add a new `DASHBOARD_E2E_*` env var to a spec. Add a scene to `_fixtures/auth.ts` and reference it by name.
- **Do not** copy `function seedSession()` from another spec. The legacy specs still have inline copies; new specs use the fixture. The legacy copies will be removed in a mechanical sweep after Solutions 3 + 4 land (see brief).
- **Do not** rely on `playwright/.auth/anonymous.json`. The current `auth.setup.ts` writes an anonymous state that no spec actually uses; storage-state-based personas arrive with Solution 4.

## Troubleshooting

| symptom                                                              | fix                                                                                                            |
| -------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `TEST_DATABASE_URL or DATABASE_URL is required`                      | `make db-up-test`, then re-run via `make test-e2e` (it sources `.env.test`).                                   |
| `local Postgres at localhost:55433 rejected the configured ... credentials` | The container is using stale data. `make db-down-test && make db-up-test`.                                |
| `signIn: persona 'outsider' was not minted`                          | Add `scenes: ["treeRefs"]` to the `seed({...})` call.                                                          |
| Spec passes locally but the dev server log shows OAuth redirects     | You forgot `signIn(page, seeded, ...)`. Without it, the fixture seeded the DB but never attached a cookie.    |
| `cargo run` rebuilds every spec                                      | Expected — incremental cache amortizes after the first run. The seeder rewrite (Solution 3) addresses this.   |

## Pointers

- Fixture source: `web/tests/e2e/_fixtures/auth.ts`
- Demo spec: `web/tests/e2e/dashboard-fixture-smoke.spec.ts`
- Seeder: `crates/api/examples/dashboard_e2e_seed.rs`
- Test DB infra: `docker-compose.test.yml`, `.env.test`, `Makefile` (`db-up-test` / `db-down-test`)
- Standardization brief (full context): see the session that introduced this file.
