import { expect, test } from "@playwright/test";

test("anonymous dashboard requests redirect to the login card", async ({
  page,
}) => {
  await page.goto("/dashboard");

  await expect(page).toHaveURL(/\/login\?next=%2Fdashboard$/);
  await expect(
    page.getByRole("heading", { name: "Sign in to opengithub" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Continue with Google" }),
  ).toBeVisible();
});

test("protected UI routes preserve the requested path in login redirects", async ({
  page,
}) => {
  await page.goto("/dashboard?tab=activity");
  await expect(page).toHaveURL(/\/login\?next=%2Fdashboard%3Ftab%3Dactivity$/);

  await page.goto("/new");
  await expect(page).toHaveURL(/\/login\?next=%2Fnew$/);

  await page.goto("/octo/example/settings/hooks");
  await expect(page).toHaveURL(
    /\/login\?next=%2Focto%2Fexample%2Fsettings%2Fhooks$/,
  );
});

test("public sign-in CTA opens the shared Google-only login page", async ({
  page,
}) => {
  await page.goto("/");

  const signIn = page.getByRole("link", { exact: true, name: "Sign in" });
  await expect(signIn).toHaveAttribute("href", "/login");
  await signIn.click();

  await expect(page).toHaveURL("http://localhost:3015/login");
  await expect(
    page.getByRole("heading", { name: "Sign in to opengithub" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Continue with Google" }),
  ).toHaveAttribute(
    "href",
    "http://localhost:3016/api/auth/google/start?next=%2Fdashboard",
  );
});

test("login page renders only the Google OAuth option and callback errors", async ({
  page,
}) => {
  await page.goto("/login?error=oauth_failed&next=/dashboard");

  await expect(
    page.getByText("Google sign-in could not be completed"),
  ).toBeVisible();
  await expect(
    page.getByText("Google sign-in could not be completed"),
  ).toContainText("Google sign-in could not be completed");
  await expect(
    page.getByRole("link", { name: "Continue with Google" }),
  ).toHaveAttribute(
    "href",
    "http://localhost:3016/api/auth/google/start?next=%2Fdashboard",
  );
  await expect(page.getByLabel(/email/i)).toHaveCount(0);
  await expect(page.getByLabel(/password/i)).toHaveCount(0);
  await expect(page.getByRole("button", { name: /sign in/i })).toHaveCount(0);
});

test("OAuth start redirects to Google with a sanitized next path", async ({
  request,
}) => {
  const response = await request.get(
    "http://localhost:3016/api/auth/google/start?next=https://evil.example/path",
    { maxRedirects: 0 },
  );

  expect(response.status()).toBe(302);
  const location = response.headers().location;
  expect(location).toContain("https://accounts.google.com/o/oauth2/v2/auth");
  expect(location).toContain("client_id=");
  expect(location).toContain("scope=openid+email+profile");
  expect(location).not.toContain("evil.example");
});

test("callback and logout failures return users to safe pages", async ({
  page,
}) => {
  await page.goto(
    "http://localhost:3016/api/auth/google/callback?error=access_denied",
  );
  await expect(page).toHaveURL(
    "http://localhost:3015/login?error=oauth_failed",
  );
  await expect(
    page.getByText("Google sign-in could not be completed"),
  ).toBeVisible();
  await expect(
    page.getByText("Google sign-in could not be completed"),
  ).toContainText("Google sign-in could not be completed");

  await page.goto("/logout");
  await expect(page).toHaveURL("http://localhost:3015/");
  await expect(
    page.getByRole("link", { exact: true, name: "Sign in" }),
  ).toBeVisible();

  await page.goto("/dashboard");
  await expect(page).toHaveURL(/\/login\?next=%2Fdashboard$/);
});

test("protected API routes return JSON auth-wall envelopes", async ({
  request,
}) => {
  const currentUser = await request.get(
    "http://localhost:3016/api/auth/current-user",
    {
      headers: {
        authorization: "Bearer not-a-real-token",
        cookie: "__Host-session=not-a-valid-session",
      },
    },
  );

  expect(currentUser.status()).toBe(401);
  expect(currentUser.headers()["content-type"]).toContain("application/json");
  const currentUserBody = await currentUser.json();
  expect(currentUserBody).toMatchObject({
    error: { code: "not_authenticated" },
    status: 401,
  });
  expect(JSON.stringify(currentUserBody)).not.toContain("not-a-valid-session");

  const repos = await request.get("http://localhost:3016/api/repos");
  expect(repos.status()).toBe(401);
  const reposBody = await repos.json();
  expect(reposBody).toMatchObject({
    error: { code: "not_authenticated" },
    status: 401,
  });
});
