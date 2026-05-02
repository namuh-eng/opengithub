import type {
  FontSizePreference,
  ThemePreference,
  UserAppearanceSettings,
} from "@/lib/api";

export const THEME_OPTIONS: {
  value: ThemePreference;
  label: string;
  description: string;
}[] = [
  {
    value: "light",
    label: "Light",
    description: "Editorial paper surfaces with dark ink.",
  },
  {
    value: "dark",
    label: "Dark",
    description: "Warm low-glare dark surfaces.",
  },
  {
    value: "system",
    label: "System",
    description: "Follow this device's light or dark setting.",
  },
  {
    value: "dark_dimmed",
    label: "Dark dimmed",
    description: "Softer dark contrast for long sessions.",
  },
  {
    value: "dark_high_contrast",
    label: "Dark high contrast",
    description: "Maximum contrast with Editorial spacing intact.",
  },
];

export const FONT_SIZE_OPTIONS: {
  value: FontSizePreference;
  label: string;
  description: string;
}[] = [
  {
    value: "small",
    label: "Small",
    description: "Dense lists with a 13px base.",
  },
  {
    value: "medium",
    label: "Medium",
    description: "Default 14px Editorial body size.",
  },
  {
    value: "large",
    label: "Large",
    description: "Roomier 16px base for readability.",
  },
];

export function normalizeTheme(
  value: string | null | undefined,
): ThemePreference {
  const normalized = value?.trim().toLowerCase().replaceAll("-", "_");
  return THEME_OPTIONS.some((option) => option.value === normalized)
    ? (normalized as ThemePreference)
    : "system";
}

export function normalizeFontSize(
  value: string | null | undefined,
): FontSizePreference {
  const normalized = value?.trim().toLowerCase();
  return FONT_SIZE_OPTIONS.some((option) => option.value === normalized)
    ? (normalized as FontSizePreference)
    : "medium";
}

export function themeAttributes(preference: ThemePreference) {
  if (preference === "light") {
    return { colorMode: "light", lightTheme: "light", darkTheme: "dark" };
  }
  if (preference === "system") {
    return { colorMode: "auto", lightTheme: "light", darkTheme: "dark" };
  }
  return { colorMode: "dark", lightTheme: "light", darkTheme: preference };
}

export function appearanceFromCookieAndSettings(
  cookieValue: string | undefined,
  fontCookieValue: string | undefined,
  settings: UserAppearanceSettings | null,
) {
  return {
    theme: settings?.theme ?? normalizeTheme(cookieValue),
    fontSize: settings?.fontSize ?? normalizeFontSize(fontCookieValue),
  };
}
