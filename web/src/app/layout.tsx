import type { Metadata } from "next";
import { Fraunces, Inter_Tight, JetBrains_Mono } from "next/font/google";
import { cookies } from "next/headers";
import Script from "next/script";
import "./globals.css";

const fraunces = Fraunces({
  variable: "--font-fraunces",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  display: "swap",
});

const interTight = Inter_Tight({
  variable: "--font-inter-tight",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-jetbrains-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "opengithub",
  description: "A calmer place for code to live.",
};

const THEME_CLASSES = {
  light: "",
  dark: "theme-dark",
  system: "",
  dark_dimmed: "theme-dark theme-dimmed",
  dark_high_contrast: "theme-dark theme-high-contrast",
} as const;

const FONT_SIZE_CLASSES = {
  small: "font-size-small",
  default: "",
  large: "font-size-large",
} as const;

type ThemeCookie = keyof typeof THEME_CLASSES;
type FontSizeCookie = keyof typeof FONT_SIZE_CLASSES;

function themeFromCookie(value: string | undefined): ThemeCookie {
  return value === "light" ||
    value === "dark" ||
    value === "system" ||
    value === "dark_dimmed" ||
    value === "dark_high_contrast"
    ? value
    : "system";
}

function fontSizeFromCookie(value: string | undefined): FontSizeCookie {
  return value === "small" || value === "large" || value === "default"
    ? value
    : "default";
}

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const cookieStore = await cookies();
  const colorMode = themeFromCookie(cookieStore.get("color_mode")?.value);
  const fontSize = fontSizeFromCookie(cookieStore.get("font_size")?.value);
  const bodyThemeClass = THEME_CLASSES[colorMode];
  const bodyFontSizeClass = FONT_SIZE_CLASSES[fontSize];

  return (
    <html
      data-color-mode={colorMode}
      data-dark-theme={
        colorMode === "dark_dimmed"
          ? "dark_dimmed"
          : colorMode === "dark_high_contrast"
            ? "dark_high_contrast"
            : "dark"
      }
      data-font-size={fontSize}
      data-light-theme="light"
      lang="en"
      className={`${fraunces.variable} ${interTight.variable} ${jetbrainsMono.variable} h-full antialiased`}
      suppressHydrationWarning
    >
      <body
        className={`og-app flex min-h-full flex-col ${bodyThemeClass} ${bodyFontSizeClass}`.trim()}
        suppressHydrationWarning
      >
        <Script id="theme-bootstrap" strategy="beforeInteractive">
          {`
(() => {
  const validThemes = new Set(["light", "dark", "system", "dark_dimmed", "dark_high_contrast"]);
  const validSizes = new Set(["small", "default", "large"]);
  const cookies = Object.fromEntries(document.cookie.split(";").map((part) => {
    const index = part.indexOf("=");
    if (index === -1) return [part.trim(), ""];
    return [part.slice(0, index).trim(), decodeURIComponent(part.slice(index + 1))];
  }));
  const mode = validThemes.has(cookies.color_mode) ? cookies.color_mode : "system";
  const fontSize = validSizes.has(cookies.font_size) ? cookies.font_size : "default";
  const resolvedDark = mode === "dark" || mode === "dark_dimmed" || mode === "dark_high_contrast" || (mode === "system" && window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.colorMode = mode;
  document.documentElement.dataset.lightTheme = "light";
  document.documentElement.dataset.darkTheme = mode === "dark_dimmed" ? "dark_dimmed" : mode === "dark_high_contrast" ? "dark_high_contrast" : "dark";
  document.documentElement.dataset.fontSize = fontSize;
  document.body.classList.toggle("theme-dark", resolvedDark);
  document.body.classList.toggle("theme-dimmed", mode === "dark_dimmed");
  document.body.classList.toggle("theme-high-contrast", mode === "dark_high_contrast");
  document.body.classList.toggle("font-size-small", fontSize === "small");
  document.body.classList.toggle("font-size-large", fontSize === "large");
})();
          `}
        </Script>
        {children}
      </body>
    </html>
  );
}
