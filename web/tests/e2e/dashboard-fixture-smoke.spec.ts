// Smoke test that proves the _fixtures/auth contract end-to-end.
// If this spec passes locally, any agent can copy its 10-line shape
// to write a new signed-in spec without touching seeder env vars.
//
// What it covers:
//   - seed() runs the seeder and returns a typed SeedResult
//   - signIn(page, seeded) attaches the owner cookie via storage state
//   - the dashboard renders for a real signed-in user
//   - Editorial non-negotiables hold (no dead controls, no horizontal overflow)

import {
  expect,
  expectNoDeadControls,
  expectNoHorizontalOverflow,
  skipWithoutTestDb,
  test,
} from "./_fixtures/auth";

test.skip(skipWithoutTestDb(), "needs TEST_DATABASE_URL or DATABASE_URL");

test("fixture: signed-in dashboard renders for the owner persona", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await signIn(page, seeded, "owner");

  await page.goto("/dashboard");
  await expect(
    page.getByRole("heading", { name: "Top repositories" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { exact: true, name: "Dashboard feed" }),
  ).toBeVisible();

  await expectNoDeadControls(page);
  await expectNoHorizontalOverflow(page);
});

test("fixture: signIn throws for an unseeded persona", async ({
  page,
  seed,
  signIn,
}) => {
  const seeded = await seed();
  await expect(async () => signIn(page, seeded, "outsider")).rejects.toThrow(
    /persona 'outsider' was not minted/,
  );
});
