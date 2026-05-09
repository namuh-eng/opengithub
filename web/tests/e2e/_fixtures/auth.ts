// Single-source Playwright fixture for opengithub E2E.
//
// New specs MUST import from this file:
//
//     import { test, expect, expectNoDeadControls } from "./_fixtures/auth";
//
// Do NOT call `cargo run --example dashboard_e2e_seed` directly from a spec —
// the fixture is the only place that knows how to translate scene names into
// the seeder's env-var contract. When the seeder is rewritten to take JSON
// stdin (Solution 3 of the E2E standardization brief), this file becomes the
// only file that has to change; specs keep working.
//
// Persona contract today (mapped to the legacy SeedOutput fields):
//   owner        — primary "Dashboard Tester" account, owns the seeded repos
//   collaborator — non-owner "Profile Action Viewer", repo collaborator
//   outsider     — read-only viewer; only minted when scene "treeRefs" runs
//
// Calling signIn(page, seeded, "outsider") without a scene that mints the
// outsider cookie throws — explicit failure beats a silent anonymous session.

import { execFileSync } from "node:child_process";
import {
  test as base,
  expect,
  type Page,
  type TestInfo,
} from "@playwright/test";

export { expect };

export type Persona = "owner" | "collaborator" | "outsider";

// Named scenes — string identifiers the spec author writes, mapped to the
// legacy env-var switches inside this file. Keep in sync with
// crates/api/examples/dashboard_e2e_seed.rs.
const SCENE_ENVS = {
  empty: { DASHBOARD_E2E_EMPTY: "1" },
  treeRefs: { DASHBOARD_E2E_TREE_REFS: "1" },
  forkRefs: { DASHBOARD_E2E_FORK_REFS: "1" },
  blobEdge: { DASHBOARD_E2E_BLOB_EDGE: "1" },
  pullRequestMerge: { PULL_REQUEST_MERGE_E2E: "1" },
  actionsRunDetail: { ACTIONS_RUN_DETAIL_E2E: "1" },
  issueTemplate: { ISSUE_TEMPLATE_E2E: "1" },
  orgProfile: { ORG_PROFILE_E2E: "1" },
  ownerPackages: { OWNER_PACKAGES_E2E: "1" },
  accountSecurity: { ACCOUNT_SECURITY_E2E: "1" },
} as const satisfies Record<string, Record<string, string>>;

export type Scene = keyof typeof SCENE_ENVS;

export type SeedSpec = {
  scenes?: Scene[];
  searchMarker?: string;
};

export type SeedResult = {
  cookieName: string;
  cookies: Record<Persona, string>;
  hrefs: {
    firstRepository: string;
    secondRepository: string;
    privateProfile: string;
    socialSourceRepository: string;
    treeRepository: string;
    trafficReadOnlyRepository: string;
    forkCompare: string;
    pullRequestMerge: string;
    actionsRunDetail: string;
    actionsJobLog: string;
    organizationProfile: string;
    organizationEmptyTeams: string;
    repositoryWiki: string;
  };
  raw: Record<string, unknown>;
};

type RawSeedOutput = {
  cookieName: string;
  cookieValue: string;
  profileActionCookieValue: string;
  trafficReadOnlyCookieValue: string;
  firstRepositoryHref: string;
  secondRepositoryHref: string;
  privateProfileHref: string;
  socialSourceRepositoryHref: string;
  treeRepositoryHref: string;
  trafficReadOnlyRepositoryHref: string;
  forkCompareHref: string;
  pullRequestMergeHref: string;
  actionsRunDetailHref: string;
  actionsJobLogHref: string;
  organizationProfileHref: string;
  organizationEmptyTeamsHref: string;
  repositoryWikiHref: string;
};

const databaseUrl = () =>
  process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL;

export const requireTestDatabase = (): string => {
  const url = databaseUrl();
  if (!url) {
    throw new Error(
      "TEST_DATABASE_URL or DATABASE_URL must be set. Run `make db-up-test` and source .env.test, " +
        "or invoke the suite via `make test-e2e` which loads .env.test for you.",
    );
  }
  return url;
};

const runSeeder = (spec: SeedSpec): RawSeedOutput => {
  requireTestDatabase();
  const sceneEnv = (spec.scenes ?? []).reduce<Record<string, string>>(
    (acc, scene) => Object.assign(acc, SCENE_ENVS[scene]),
    {},
  );
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    SESSION_COOKIE_NAME: "og_session",
    ...sceneEnv,
  };
  if (spec.searchMarker !== undefined) {
    env.SEARCH_E2E_MARKER = spec.searchMarker;
  }
  const stdout = execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "-p",
      "opengithub-api",
      "--example",
      "dashboard_e2e_seed",
    ],
    { cwd: "..", env },
  ).toString();
  return JSON.parse(stdout) as RawSeedOutput;
};

const toSeedResult = (raw: RawSeedOutput): SeedResult => ({
  cookieName: raw.cookieName,
  cookies: {
    owner: raw.cookieValue,
    collaborator: raw.profileActionCookieValue,
    outsider: raw.trafficReadOnlyCookieValue,
  },
  hrefs: {
    firstRepository: raw.firstRepositoryHref,
    secondRepository: raw.secondRepositoryHref,
    privateProfile: raw.privateProfileHref,
    socialSourceRepository: raw.socialSourceRepositoryHref,
    treeRepository: raw.treeRepositoryHref,
    trafficReadOnlyRepository: raw.trafficReadOnlyRepositoryHref,
    forkCompare: raw.forkCompareHref,
    pullRequestMerge: raw.pullRequestMergeHref,
    actionsRunDetail: raw.actionsRunDetailHref,
    actionsJobLog: raw.actionsJobLogHref,
    organizationProfile: raw.organizationProfileHref,
    organizationEmptyTeams: raw.organizationEmptyTeamsHref,
    repositoryWiki: raw.repositoryWikiHref,
  },
  raw: raw as unknown as Record<string, unknown>,
});

export type Fixtures = {
  seed: (spec?: SeedSpec) => Promise<SeedResult>;
  signIn: (page: Page, seeded: SeedResult, as?: Persona) => Promise<void>;
};

export const test = base.extend<Fixtures>({
  // biome-ignore lint/correctness/noEmptyPattern: Playwright requires destructuring on the first arg
  seed: async ({}, use: (fn: Fixtures["seed"]) => Promise<void>) => {
    await use(async (spec = {}) => toSeedResult(runSeeder(spec)));
  },
  // biome-ignore lint/correctness/noEmptyPattern: Playwright requires destructuring on the first arg
  signIn: async ({}, use: (fn: Fixtures["signIn"]) => Promise<void>) => {
    await use(async (page, seeded, who: Persona = "owner") => {
      const value = seeded.cookies[who];
      if (!value) {
        throw new Error(
          `signIn: persona '${who}' was not minted. ` +
            (who === "outsider"
              ? "The outsider cookie requires the 'treeRefs' scene."
              : "Add the appropriate scene to seed({ scenes: [...] })."),
        );
      }
      await page.context().addCookies([
        {
          name: seeded.cookieName,
          value,
          domain: "localhost",
          path: "/",
          httpOnly: true,
          sameSite: "Lax",
          secure: false,
        },
      ]);
    });
  },
});

// Skip helper for specs that cannot run without a real test DB.
// Place at the top of a spec file:  test.skip(skipWithoutTestDb(), "..");
export const skipWithoutTestDb = () => !databaseUrl();

// Editorial design system non-negotiables, applied per page.
// Specs should call this on every page they assert against.
export const expectNoDeadControls = async (page: Page): Promise<void> => {
  await expect(page.locator('a[href="#"], a:not([href])')).toHaveCount(0);
  for (const button of await page.locator("button:visible").all()) {
    await expect(button).toHaveAccessibleName(/.+/);
  }
};

export const expectNoHorizontalOverflow = async (page: Page): Promise<void> => {
  const metrics = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth);
};

// Shared screenshot path convention from CLAUDE.md / project memory.
// Use:  await page.screenshot({ path: screenshotPath(testInfo, "dashboard") });
export const screenshotPath = (testInfo: TestInfo, label: string): string =>
  `${testInfo.project.outputDir}/../../ralph/screenshots/build/${label}.jpg`.replace(
    /\\/g,
    "/",
  );
