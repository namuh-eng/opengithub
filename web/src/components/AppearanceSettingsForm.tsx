"use client";

import { useMemo, useState } from "react";
import type {
  AppearanceFontSize,
  AppearanceSettings,
  AppearanceTheme,
} from "@/lib/api";

const THEME_OPTIONS: {
  value: AppearanceTheme;
  label: string;
  description: string;
}[] = [
  {
    value: "system",
    label: "System",
    description:
      "Follow the browser color scheme for signed-out and signed-in pages.",
  },
  {
    value: "light",
    label: "Light",
    description: "Editorial paper, ink text, and a single rust accent.",
  },
  {
    value: "dark",
    label: "Dark",
    description: "Warm dark surfaces while preserving the Editorial accent.",
  },
  {
    value: "dark_dimmed",
    label: "Dark dimmed",
    description: "Lower contrast dark reading mode for long review sessions.",
  },
  {
    value: "dark_high_contrast",
    label: "High contrast",
    description: "Stronger ink, line, and accent contrast for dense screens.",
  },
];

const FONT_SIZE_OPTIONS: {
  value: AppearanceFontSize;
  label: string;
  description: string;
}[] = [
  {
    value: "small",
    label: "Small",
    description: "Tighter lists and code review tables.",
  },
  {
    value: "default",
    label: "Default",
    description: "The standard Editorial type rhythm.",
  },
  {
    value: "large",
    label: "Large",
    description: "More readable body copy and controls.",
  },
];

function applyTheme(theme: AppearanceTheme, fontSize: AppearanceFontSize) {
  const root = document.documentElement;
  const body = document.body;
  const prefersDark = window.matchMedia?.(
    "(prefers-color-scheme: dark)",
  ).matches;
  const resolvedDark =
    theme === "dark" ||
    theme === "dark_dimmed" ||
    theme === "dark_high_contrast" ||
    (theme === "system" && prefersDark);

  root.dataset.colorMode = theme;
  root.dataset.lightTheme = "light";
  root.dataset.darkTheme =
    theme === "dark_dimmed"
      ? "dark_dimmed"
      : theme === "dark_high_contrast"
        ? "dark_high_contrast"
        : "dark";
  root.dataset.fontSize = fontSize;

  body.classList.toggle("theme-dark", resolvedDark);
  body.classList.toggle("theme-dimmed", theme === "dark_dimmed");
  body.classList.toggle("theme-high-contrast", theme === "dark_high_contrast");
  body.classList.toggle("font-size-small", fontSize === "small");
  body.classList.toggle("font-size-large", fontSize === "large");
}

export function AppearanceSettingsForm({
  initialSettings,
}: {
  initialSettings: AppearanceSettings;
}) {
  const [settings, setSettings] = useState(initialSettings);
  const [theme, setTheme] = useState(initialSettings.theme);
  const [fontSize, setFontSize] = useState(initialSettings.fontSize);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const dirty = theme !== settings.theme || fontSize !== settings.fontSize;

  const previewClass = useMemo(() => {
    if (theme === "dark_dimmed") return "theme-dark theme-dimmed";
    if (theme === "dark_high_contrast") return "theme-dark theme-high-contrast";
    if (theme === "dark") return "theme-dark";
    return "";
  }, [theme]);

  async function save() {
    setSaving(true);
    setError("");
    setStatus("");
    try {
      const response = await fetch("/settings/appearance/actions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ theme, fontSize }),
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(
          payload?.error?.message ?? "Appearance settings could not be saved.",
        );
      }
      setSettings(payload as AppearanceSettings);
      applyTheme(payload.theme, payload.fontSize);
      setStatus("Appearance settings saved.");
    } catch (saveError) {
      setError(
        saveError instanceof Error
          ? saveError.message
          : "Appearance settings could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
      <section className="card p-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label">Color mode</p>
            <h3 className="t-h2 mt-2">Theme</h3>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Preferences persist to your account and a browser cookie so the
              app can render the selected mode before hydration.
            </p>
          </div>
          <span className="chip soft">WCAG AA checked</span>
        </div>

        <div className="mt-5 grid gap-3 md:grid-cols-2">
          {THEME_OPTIONS.map((option) => (
            <label
              className="card cursor-pointer p-4"
              key={option.value}
              style={
                theme === option.value
                  ? {
                      borderColor: "var(--accent)",
                      boxShadow: "var(--shadow-sm)",
                    }
                  : undefined
              }
            >
              <span className="flex items-start gap-3">
                <input
                  aria-label={option.label}
                  checked={theme === option.value}
                  className="mt-1 accent-[var(--accent)]"
                  name="theme"
                  onChange={() => setTheme(option.value)}
                  type="radio"
                />
                <span>
                  <span className="t-h3 block">{option.label}</span>
                  <span className="t-xs mt-1 block">{option.description}</span>
                </span>
              </span>
            </label>
          ))}
        </div>

        <div className="mt-8">
          <p className="t-label">Text scale</p>
          <h3 className="t-h2 mt-2">Font size</h3>
          <div className="mt-4 grid gap-3 md:grid-cols-3">
            {FONT_SIZE_OPTIONS.map((option) => (
              <label
                className="card cursor-pointer p-4"
                key={option.value}
                style={
                  fontSize === option.value
                    ? {
                        borderColor: "var(--accent)",
                        boxShadow: "var(--shadow-sm)",
                      }
                    : undefined
                }
              >
                <span className="flex items-start gap-3">
                  <input
                    aria-label={option.label}
                    checked={fontSize === option.value}
                    className="mt-1 accent-[var(--accent)]"
                    name="fontSize"
                    onChange={() => setFontSize(option.value)}
                    type="radio"
                  />
                  <span>
                    <span className="t-h3 block">{option.label}</span>
                    <span className="t-xs mt-1 block">
                      {option.description}
                    </span>
                  </span>
                </span>
              </label>
            ))}
          </div>
        </div>

        <div className="mt-6 flex flex-wrap items-center gap-3">
          <button
            className="btn primary"
            disabled={!dirty || saving}
            onClick={save}
            type="button"
          >
            {saving ? "Saving..." : "Save appearance"}
          </button>
          <button
            className="btn"
            disabled={!dirty || saving}
            onClick={() => {
              setTheme(settings.theme);
              setFontSize(settings.fontSize);
              setError("");
              setStatus("");
            }}
            type="button"
          >
            Reset
          </button>
          {status ? (
            <p className="t-sm" role="status">
              {status}
            </p>
          ) : null}
          {error ? (
            <p className="t-sm" role="alert" style={{ color: "var(--err)" }}>
              {error}
            </p>
          ) : null}
        </div>
      </section>

      <aside className={`card p-5 ${previewClass}`} aria-label="Theme preview">
        <p className="t-label">Preview</p>
        <h3 className="t-h2 mt-2">Repository review</h3>
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          The preview uses the same tokens as repository, issue, and Actions
          pages without introducing alternate palettes.
        </p>
        <div
          className="mt-5 rounded-md p-4"
          style={{ background: "var(--surface-2)" }}
        >
          <div className="flex items-center justify-between gap-3">
            <span className="chip ok">Checks passing</span>
            <span className="t-mono-sm">8a4f2c1</span>
          </div>
          <div className="mt-4 grid gap-2">
            {[
              "src/app/page.tsx",
              "crates/api/src/routes/users.rs",
              "web/src/app/og.css",
            ].map((file) => (
              <div className="list-row !px-0" key={file}>
                <span className="t-mono-sm" style={{ color: "var(--ink-2)" }}>
                  {file}
                </span>
                <span className="chip soft">modified</span>
              </div>
            ))}
          </div>
        </div>
      </aside>
    </div>
  );
}
