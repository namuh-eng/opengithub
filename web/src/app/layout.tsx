import type { Metadata } from "next";
import { Fraunces, Inter_Tight, JetBrains_Mono } from "next/font/google";
import { cookies, headers } from "next/headers";
import { ThemeClientScript } from "@/components/ThemeClientScript";
import { getUserAppearanceSettingsFromCookie } from "@/lib/api";
import {
  appearanceFromCookieAndSettings,
  normalizeFontSize,
  normalizeTheme,
  themeAttributes,
} from "@/lib/theme";
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

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return <RootLayoutContent>{children}</RootLayoutContent>;
}

async function RootLayoutContent({ children }: { children: React.ReactNode }) {
  const [cookieStore, requestHeaders] = await Promise.all([
    cookies(),
    headers(),
  ]);
  const cookieHeader = requestHeaders.get("cookie");
  const settings = await getUserAppearanceSettingsFromCookie(cookieHeader);
  const { theme, fontSize } = appearanceFromCookieAndSettings(
    cookieStore.get("color_mode")?.value,
    cookieStore.get("font_size")?.value,
    settings,
  );
  const attrs = themeAttributes(normalizeTheme(theme));
  const normalizedFontSize = normalizeFontSize(fontSize);

  return (
    <html
      data-color-mode={attrs.colorMode}
      data-dark-theme={attrs.darkTheme}
      data-font-size={normalizedFontSize}
      data-light-theme={attrs.lightTheme}
      lang="en"
      className={`${fraunces.variable} ${interTight.variable} ${jetbrainsMono.variable} h-full antialiased`}
    >
      <body className="og-app flex min-h-full flex-col">
        <ThemeClientScript fontSize={normalizedFontSize} />
        {children}
      </body>
    </html>
  );
}
