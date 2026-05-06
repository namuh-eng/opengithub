import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppearanceSettingsForm } from "@/components/AppearanceSettingsForm";
import type { AppearanceSettings } from "@/lib/api";

const settings: AppearanceSettings = {
  userId: "user-1",
  theme: "system",
  fontSize: "default",
  updatedAt: "2026-05-07T00:00:00Z",
};

afterEach(() => {
  vi.restoreAllMocks();
  document.documentElement.dataset.colorMode = "";
  document.documentElement.dataset.fontSize = "";
  document.body.className = "";
});

describe("AppearanceSettingsForm", () => {
  it("renders theme, font-size controls, and a token-backed preview", () => {
    render(<AppearanceSettingsForm initialSettings={settings} />);

    expect(screen.getByRole("heading", { name: "Theme" })).toBeVisible();
    expect(screen.getByLabelText("System")).toBeChecked();
    expect(screen.getByLabelText("Dark dimmed")).toBeVisible();
    expect(screen.getByLabelText("High contrast")).toBeVisible();
    expect(screen.getByLabelText("Default")).toBeChecked();
    expect(screen.getByLabelText("Theme preview")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Save appearance" }),
    ).toBeDisabled();
  });

  it("saves preferences through the same-origin action and applies first-paint attributes", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        ...settings,
        theme: "dark_high_contrast",
        fontSize: "large",
      }),
    });
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => ({ matches: false })),
    );
    render(<AppearanceSettingsForm initialSettings={settings} />);

    fireEvent.click(screen.getByLabelText("High contrast"));
    fireEvent.click(screen.getByLabelText("Large"));
    fireEvent.click(screen.getByRole("button", { name: "Save appearance" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/appearance/actions",
        expect.objectContaining({ method: "PATCH" }),
      ),
    );
    const payload = JSON.parse(fetchMock.mock.calls[0][1].body as string);
    expect(payload).toEqual({
      theme: "dark_high_contrast",
      fontSize: "large",
    });
    expect(document.documentElement.dataset.colorMode).toBe(
      "dark_high_contrast",
    );
    expect(document.documentElement.dataset.darkTheme).toBe(
      "dark_high_contrast",
    );
    expect(document.body.classList.contains("theme-dark")).toBe(true);
    expect(document.body.classList.contains("theme-high-contrast")).toBe(true);
    expect(document.body.classList.contains("font-size-large")).toBe(true);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Appearance settings saved.",
    );
  });

  it("has no inert links or unnamed buttons", () => {
    const { container } = render(
      <AppearanceSettingsForm initialSettings={settings} />,
    );

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });
});
