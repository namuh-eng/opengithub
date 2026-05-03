import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PersonalProfileSettingsForm } from "@/components/PersonalProfileSettingsForm";
import type { PersonalProfileSettings } from "@/lib/api";

const settings: PersonalProfileSettings = {
  userId: "user-1",
  login: "mona",
  displayName: "Mona Lisa",
  publicEmailId: "email-1",
  publicEmail: "mona@example.com",
  emails: [
    {
      id: "email-1",
      email: "mona@example.com",
      isPrimary: true,
      isPublic: true,
      verified: true,
    },
  ],
  bio: "Building in public",
  pronouns: "she/her",
  websiteUrl: "https://example.com",
  company: "NamuH",
  location: "Seoul",
  displayLocalTime: true,
  timeZone: "Asia/Seoul",
  privateProfile: false,
  showPrivateContributionCount: false,
  achievementsEnabled: true,
  preferredLanguage: "en",
  socialAccounts: [
    { provider: "x", handleOrUrl: "@mona", position: 1 },
    { provider: "mastodon", handleOrUrl: "", position: 2 },
    { provider: "linkedin", handleOrUrl: "", position: 3 },
    { provider: "bluesky", handleOrUrl: "", position: 4 },
  ],
  avatar: null,
  updatedAt: "2026-05-03T00:00:00Z",
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("PersonalProfileSettingsForm", () => {
  it("renders all required public profile fields and starts with update disabled", () => {
    render(<PersonalProfileSettingsForm initialSettings={settings} />);

    expect(
      screen.getByRole("heading", { name: "Profile details" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Name")).toHaveValue("Mona Lisa");
    expect(screen.getByLabelText("Public email")).toHaveValue("email-1");
    expect(screen.getByLabelText("Bio")).toHaveValue("Building in public");
    expect(screen.getByLabelText("Pronouns")).toHaveValue("she/her");
    expect(screen.getByLabelText("URL")).toHaveValue("https://example.com");
    expect(screen.getByLabelText("Company")).toHaveValue("NamuH");
    expect(screen.getByLabelText("Location")).toHaveValue("Seoul");
    expect(screen.getByLabelText("Display current local time")).toBeChecked();
    expect(screen.getByLabelText("Time zone")).toHaveValue("Asia/Seoul");
    expect(screen.getByLabelText("Preferred language")).toHaveValue("en");
    expect(
      screen.getByRole("button", { name: "Update profile" }),
    ).toBeDisabled();

    const socialRegion = screen.getByText("Social accounts").closest("div");
    expect(socialRegion).not.toBeNull();
    expect(screen.getByDisplayValue("@mona")).toBeVisible();
  });

  it("enables and saves changed text fields with optional fields cleared", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ ...settings, displayName: "Mona", bio: "" }),
    });
    vi.stubGlobal("fetch", fetchMock);
    render(<PersonalProfileSettingsForm initialSettings={settings} />);

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Mona" },
    });
    fireEvent.change(screen.getByLabelText("Bio"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Update profile" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/profile/actions",
        expect.objectContaining({ method: "PATCH" }),
      ),
    );
    const payload = JSON.parse(fetchMock.mock.calls[0][1].body as string);
    expect(payload.displayName).toBe("Mona");
    expect(payload.bio).toBe("");
    expect(screen.getByRole("status")).toHaveTextContent(
      "Public profile updated",
    );
  });

  it("shows inline validation for invalid URL before saving", () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    render(<PersonalProfileSettingsForm initialSettings={settings} />);

    fireEvent.change(screen.getByLabelText("URL"), {
      target: { value: "example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Update profile" }));

    expect(
      screen.getByText("URL must start with http:// or https://."),
    ).toBeVisible();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("saves privacy checkboxes without leaving the page", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ ...settings, privateProfile: true }),
    });
    vi.stubGlobal("fetch", fetchMock);
    render(<PersonalProfileSettingsForm initialSettings={settings} />);

    fireEvent.click(
      screen.getByRole("checkbox", { name: /Make my profile private/ }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save privacy" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    const payload = JSON.parse(fetchMock.mock.calls[0][1].body as string);
    expect(payload).toEqual({
      privateProfile: true,
      showPrivateContributionCount: false,
      achievementsEnabled: true,
    });
    expect(screen.getByRole("status")).toHaveTextContent(
      "Profile privacy updated",
    );
  });

  it("rejects invalid avatar files inline and keeps avatar controls live", () => {
    render(<PersonalProfileSettingsForm initialSettings={settings} />);
    const input = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["not an image"], "avatar.txt", {
      type: "text/plain",
    });

    fireEvent.change(input, { target: { files: [file] } });

    expect(
      screen.getByText("Avatar must be a PNG, JPEG, WebP, or GIF image."),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Reset preview" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Remove" })).toBeDisabled();
  });

  it("has no inert links or unnamed buttons", () => {
    const { container } = render(
      <PersonalProfileSettingsForm initialSettings={settings} />,
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
    expect(screen.getByText("MO")).toBeVisible();
  });
});
