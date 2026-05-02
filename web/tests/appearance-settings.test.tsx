import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppearanceSettingsForm } from "@/components/AppearanceSettingsForm";
import type { UserAppearanceSettings } from "@/lib/api";
import { appearanceFromCookieAndSettings, themeAttributes } from "@/lib/theme";

const settings: UserAppearanceSettings = {
  userId: "user-1",
  theme: "system",
  fontSize: "medium",
  updatedAt: "2026-05-03T00:00:00Z",
};

afterEach(() => {
  vi.restoreAllMocks();
  window.__opengithubApplyTheme = undefined;
});

describe("AppearanceSettingsForm", () => {
  it("renders the theme, font size, and preview controls without dead controls", () => {
    const { container } = render(
      <AppearanceSettingsForm initialSettings={settings} />,
    );

    expect(screen.getByRole("heading", { name: "Color mode" })).toBeVisible();
    expect(screen.getByRole("radio", { name: /System/ })).toBeChecked();
    expect(screen.getByRole("radio", { name: /Medium/ })).toBeChecked();
    expect(
      screen.getByRole("button", { name: "Save appearance" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Hide preview" })).toBeEnabled();
    expect(
      screen.getByRole("heading", { name: "Repository activity preview" }),
    ).toBeVisible();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("persists changed appearance settings and applies html attributes immediately", async () => {
    const applyTheme = vi.fn();
    window.__opengithubApplyTheme = applyTheme;
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        ...settings,
        theme: "dark_dimmed",
        fontSize: "large",
      }),
    });
    vi.stubGlobal("fetch", fetchMock);
    render(<AppearanceSettingsForm initialSettings={settings} />);

    fireEvent.click(screen.getByRole("radio", { name: /Dark dimmed/ }));
    fireEvent.click(screen.getByRole("radio", { name: /Large/ }));
    fireEvent.click(screen.getByRole("button", { name: "Save appearance" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/appearance/actions",
        expect.objectContaining({ method: "PATCH" }),
      ),
    );
    const payload = JSON.parse(fetchMock.mock.calls[0][1].body as string);
    expect(payload).toEqual({ theme: "dark_dimmed", fontSize: "large" });
    expect(applyTheme).toHaveBeenCalledWith("dark_dimmed", "large");
    expect(screen.getByRole("status")).toHaveTextContent(
      "Appearance preferences saved",
    );
  });

  it("normalizes cookie fallback and maps data attributes for system visitors", () => {
    expect(
      appearanceFromCookieAndSettings("dark-high-contrast", "large", null),
    ).toEqual({
      theme: "dark_high_contrast",
      fontSize: "large",
    });
    expect(appearanceFromCookieAndSettings("neon", "tiny", settings)).toEqual({
      theme: "system",
      fontSize: "medium",
    });
    expect(themeAttributes("system")).toEqual({
      colorMode: "auto",
      lightTheme: "light",
      darkTheme: "dark",
    });
  });
});
