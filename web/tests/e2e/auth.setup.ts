import { expect, test as setup } from "@playwright/test";

const authStatePath = "playwright/.auth/anonymous.json";
const apiBaseUrl = process.env.API_URL ?? "http://localhost:3016";

setup.setTimeout(180_000);

async function waitForApiHealth() {
  const deadline = Date.now() + 180_000;
  let lastError: unknown;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${apiBaseUrl}/health`, {
        cache: "no-store",
      });
      if (response.ok) {
        return;
      }
      lastError = new Error(`health returned ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }

  throw new Error(
    `API health check did not become ready at ${apiBaseUrl}/health`,
    {
      cause: lastError,
    },
  );
}

setup("prepare anonymous auth state", async ({ page }) => {
  // Playwright's webServer readiness only waits for the Next.js port. The Rust
  // API can still be cold-compiling, but auth/API smoke tests call it directly.
  await waitForApiHealth();

  // Keep the default setup deterministic and local. A real authenticated
  // browser state requires a Postgres-backed session plus a signed Rust cookie,
  // so QA should create it through Google OAuth or a test DB bootstrap flow.
  await page.goto("/login", { waitUntil: "domcontentloaded" });
  await expect(
    page.getByRole("heading", { name: "Sign in to opengithub" }),
  ).toBeVisible({ timeout: 30_000 });
  await page.context().storageState({ path: authStatePath });
});
