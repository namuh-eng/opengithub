"use client";

import { useEffect } from "react";
import type { FontSizePreference } from "@/lib/api";

function applyResolvedTheme(root: HTMLElement, media: MediaQueryList) {
  const mode = root.dataset.colorMode || "auto";
  const darkTheme = root.dataset.darkTheme || "dark";
  const lightTheme = root.dataset.lightTheme || "light";
  root.dataset.resolvedTheme =
    mode === "auto"
      ? media.matches
        ? darkTheme
        : lightTheme
      : mode === "dark"
        ? darkTheme
        : lightTheme;
}

export function ThemeClientScript({
  fontSize,
}: {
  fontSize: FontSizePreference;
}) {
  useEffect(() => {
    const root = document.documentElement;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => applyResolvedTheme(root, media);
    root.dataset.themeReady = "true";
    root.dataset.fontSize = fontSize;
    apply();
    media.addEventListener("change", apply);
    window.__opengithubApplyTheme = (preference, nextFontSize) => {
      const nextTheme = String(preference || "system").replaceAll("-", "_");
      root.dataset.colorMode =
        nextTheme === "light"
          ? "light"
          : nextTheme === "system"
            ? "auto"
            : "dark";
      root.dataset.lightTheme = "light";
      root.dataset.darkTheme =
        nextTheme === "light" || nextTheme === "system" ? "dark" : nextTheme;
      root.dataset.fontSize = String(nextFontSize || "medium");
      apply();
    };
    return () => {
      media.removeEventListener("change", apply);
    };
  }, [fontSize]);

  return null;
}
