import { expect, test } from "@playwright/test";

test("public home navigation dropdowns and search work", async ({ page }) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: /A calmer place for code to live/i }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Product" }).click();
  await expect(page.getByRole("menu", { name: "Product menu" })).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: /Repositories/i }),
  ).toHaveAttribute("href", "/explore");

  await page.keyboard.press("Escape");
  await expect(page.getByRole("menu", { name: "Product menu" })).toHaveCount(0);

  await page.getByLabel("Search", { exact: true }).fill("repo:opengithub");
  await page.getByRole("form", { name: "Search opengithub" }).press("Enter");
  await expect(page).toHaveURL(
    /\/login\?next=%2Fsearch%3Fq%3Drepo%253Aopengithub$/,
  );
});

test("public home auth CTAs and footer links are live", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("link", { name: "Sign in" })).toHaveAttribute(
    "href",
    "/login",
  );
  await expect(
    page.getByRole("link", { exact: true, name: "Sign up" }),
  ).toHaveAttribute("href", "/login");
  await expect(page.getByRole("link", { name: "API" })).toHaveAttribute(
    "href",
    "/docs/api",
  );

  await page.getByRole("link", { exact: true, name: "Sign up" }).click();
  await expect(page).toHaveURL("http://localhost:3015/login");
  await expect(
    page.getByRole("heading", { name: "Sign in to opengithub" }),
  ).toBeVisible();
});

test("public home does not ship dead placeholder controls", async ({
  page,
}) => {
  await page.goto("/");

  const deadLinks = page.locator('a[href="#"], button:has-text("TODO")');
  await expect(deadLinks).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /A calmer place for code to live/i }),
  ).toBeVisible();
});
