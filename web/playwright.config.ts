import { defineConfig, devices } from "@playwright/test";

const port = 3015;
const chromiumExecutablePath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH;
const chromiumLaunchOptions = chromiumExecutablePath
  ? { launchOptions: { executablePath: chromiumExecutablePath } }
  : {};

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: `http://localhost:${port}`,
    trace: "retain-on-failure",
    ...chromiumLaunchOptions,
  },
  projects: [
    {
      name: "setup",
      testMatch: /auth\.setup\.ts/,
    },
    {
      name: "chromium",
      dependencies: ["setup"],
      use: {
        ...devices["Desktop Chrome"],
        ...chromiumLaunchOptions,
      },
    },
  ],
  webServer: {
    command: "cd .. && make dev",
    url: `http://localhost:${port}`,
    reuseExistingServer: true,
    timeout: 120_000,
    env: {
      APP_URL: `http://localhost:${port}`,
      PUBLIC_APP_URL: `http://localhost:${port}`,
      API_URL: "http://localhost:3016",
      AUTH_GOOGLE_ID:
        process.env.AUTH_GOOGLE_ID ??
        "playwright-client-id.apps.googleusercontent.com",
      AUTH_GOOGLE_SECRET:
        process.env.AUTH_GOOGLE_SECRET ?? "playwright-client-secret",
      SESSION_SECRET:
        process.env.SESSION_SECRET ??
        "playwright-session-secret-with-enough-entropy",
      SESSION_COOKIE_NAME: process.env.SESSION_COOKIE_NAME ?? "og_session",
      SESSION_COOKIE_SECURE: process.env.SESSION_COOKIE_SECURE ?? "false",
      DB_SSL: process.env.DB_SSL ?? "false",
      DATABASE_URL:
        process.env.TEST_DATABASE_URL ?? process.env.DATABASE_URL ?? "",
    },
  },
});
