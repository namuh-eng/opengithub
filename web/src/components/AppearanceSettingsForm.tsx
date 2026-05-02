"use client";

import { useMemo, useState } from "react";
import type {
  FontSizePreference,
  ThemePreference,
  UserAppearanceSettings,
} from "@/lib/api";
import { FONT_SIZE_OPTIONS, THEME_OPTIONS, themeAttributes } from "@/lib/theme";

type AppearanceSettingsFormProps = {
  initialSettings: UserAppearanceSettings;
};

export function AppearanceSettingsForm({
  initialSettings,
}: AppearanceSettingsFormProps) {
  const [theme, setTheme] = useState<ThemePreference>(initialSettings.theme);
  const [fontSize, setFontSize] = useState<FontSizePreference>(
    initialSettings.fontSize,
  );
  const [saved, setSaved] = useState(initialSettings);
  const [showPreview, setShowPreview] = useState(true);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const dirty = theme !== saved.theme || fontSize !== saved.fontSize;
  const previewAttrs = useMemo(() => themeAttributes(theme), [theme]);

  async function save() {
    setSaving(true);
    setError(null);
    try {
      const response = await fetch("/settings/appearance/actions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ theme, fontSize }),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Appearance settings could not be saved",
        );
      }
      const next = body as UserAppearanceSettings;
      setSaved(next);
      setTheme(next.theme);
      setFontSize(next.fontSize);
      window.__opengithubApplyTheme?.(next.theme, next.fontSize);
      setStatus("Appearance preferences saved");
    } catch (saveError) {
      setError(
        saveError instanceof Error
          ? saveError.message
          : "Appearance settings could not be saved",
      );
    } finally {
      setSaving(false);
    }
  }

  function reset() {
    setTheme(saved.theme);
    setFontSize(saved.fontSize);
    setError(null);
    setStatus("Unsaved appearance changes reset");
  }

  return (
    <div className="grid gap-6">
      {status ? (
        <div className="chip ok w-fit" role="status">
          {status}
        </div>
      ) : null}
      {error ? (
        <div className="chip err w-fit" role="alert">
          {error}
        </div>
      ) : null}

      <section className="card p-5" aria-labelledby="theme-heading">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <p className="t-label">Theme</p>
            <h3 className="t-h2 mt-2" id="theme-heading">
              Color mode
            </h3>
            <p
              className="t-body mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Choose how OpenGitHub applies Editorial tokens across every page.
              System mode follows{" "}
              <span className="t-mono-sm">prefers-color-scheme</span> for this
              device.
            </p>
          </div>
          <button
            className="btn sm"
            onClick={() => setShowPreview((value) => !value)}
            type="button"
          >
            {showPreview ? "Hide preview" : "Show preview"}
          </button>
        </div>

        <fieldset className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          <legend className="sr-only">Theme preference</legend>
          {THEME_OPTIONS.map((option) => (
            <label
              className="card flex cursor-pointer gap-3 p-4 hover:bg-[var(--hover)]"
              key={option.value}
            >
              <input
                checked={theme === option.value}
                className="mt-1 accent-[var(--accent)]"
                name="theme"
                onChange={() => setTheme(option.value)}
                type="radio"
                value={option.value}
              />
              <span>
                <span className="t-h3 block">{option.label}</span>
                <span
                  className="t-sm mt-1 block"
                  style={{ color: "var(--ink-3)" }}
                >
                  {option.description}
                </span>
              </span>
            </label>
          ))}
        </fieldset>
      </section>

      <section className="card p-5" aria-labelledby="font-size-heading">
        <p className="t-label">Accessibility</p>
        <h3 className="t-h2 mt-2" id="font-size-heading">
          Font size
        </h3>
        <p className="t-body mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
          Adjust the site-wide body scale without changing the Editorial type
          ramp or mono code font.
        </p>
        <fieldset className="mt-5 grid gap-3 md:grid-cols-3">
          <legend className="sr-only">Font size preference</legend>
          {FONT_SIZE_OPTIONS.map((option) => (
            <label
              className="card flex cursor-pointer gap-3 p-4 hover:bg-[var(--hover)]"
              key={option.value}
            >
              <input
                checked={fontSize === option.value}
                className="mt-1 accent-[var(--accent)]"
                name="fontSize"
                onChange={() => setFontSize(option.value)}
                type="radio"
                value={option.value}
              />
              <span>
                <span className="t-h3 block">{option.label}</span>
                <span
                  className="t-sm mt-1 block"
                  style={{ color: "var(--ink-3)" }}
                >
                  {option.description}
                </span>
              </span>
            </label>
          ))}
        </fieldset>
      </section>

      {showPreview ? (
        <section
          data-theme-scope="true"
          aria-labelledby="preview-heading"
          className="card p-5"
          data-color-mode={previewAttrs.colorMode}
          data-dark-theme={previewAttrs.darkTheme}
          data-font-size={fontSize}
          data-light-theme={previewAttrs.lightTheme}
          data-resolved-theme={theme === "system" ? "dark" : theme}
        >
          <p className="t-label">Show preview</p>
          <h3 className="t-h2 mt-2" id="preview-heading">
            Repository activity preview
          </h3>
          <div className="mt-5 grid gap-4 lg:grid-cols-[minmax(0,1fr)_280px]">
            <div className="card p-4">
              <div className="flex flex-wrap items-center gap-2">
                <span className="chip active">Active pull request</span>
                <span className="chip ok">Checks passing</span>
                <span className="chip warn">Review requested</span>
              </div>
              <h4 className="t-h3 mt-4">Refine responsive navigation tokens</h4>
              <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
                Preview cards use the same variables as the rest of the app, so
                contrast and spacing match the saved preference.
              </p>
            </div>
            <div className="card p-4">
              <p className="t-label">Keyboard</p>
              <p className="t-body mt-3">
                Press <span className="kbd">⌘K</span> to open navigation after
                saving.
              </p>
            </div>
          </div>
        </section>
      ) : null}

      <div className="flex flex-wrap gap-2">
        <button
          className="btn primary"
          disabled={!dirty || saving}
          onClick={save}
          type="button"
        >
          {saving ? "Saving…" : "Save appearance"}
        </button>
        <button
          className="btn"
          disabled={!dirty || saving}
          onClick={reset}
          type="button"
        >
          Reset changes
        </button>
      </div>
    </div>
  );
}
