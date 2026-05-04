import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OrganizationProfileSettingsForm } from "@/components/OrganizationProfileSettingsForm";
import { OrganizationSettingsShell } from "@/components/OrganizationSettingsShell";
import type {
  AppShellContext,
  AuthSession,
  OrganizationProfileSettings,
} from "@/lib/api";

function session(): AuthSession {
  return {
    authenticated: true,
    user: {
      avatar_url: null,
      display_name: "Dashboard Tester",
      email: "tester@example.com",
      id: "user-1",
    },
  };
}

function shellContext(): AppShellContext {
  const authUser = session().user;
  if (!authUser) {
    throw new Error("test session must include an authenticated user");
  }
  return {
    organizations: [
      {
        href: "/orgs/acme-labs",
        id: "org-1",
        displayName: "Acme Labs",
        role: "owner",
        slug: "acme-labs",
      },
    ],
    quickLinks: [],
    recentRepositories: [],
    teams: [],
    unreadNotificationCount: 0,
    user: authUser,
  };
}

function profileSettings(
  overrides: Partial<OrganizationProfileSettings> = {},
): OrganizationProfileSettings {
  const base: OrganizationProfileSettings = {
    avatar: {
      avatarUrl: null,
      storageConfigured: false,
      unavailableReason:
        "Organization avatar upload will be enabled after the S3 avatar pipeline is wired.",
      uploadAvailable: false,
    },
    organization: {
      href: "/orgs/acme-labs",
      id: "org-1",
      name: "Acme Labs",
      settingsHref: "/organizations/acme-labs/settings/profile",
      slug: "acme-labs",
    },
    profile: {
      billingEmail: "billing@example.com",
      companyName: "Acme Inc.",
      contactEmail: "admin@example.com",
      description: "Builds quieter developer tools.",
      displayName: "Acme Labs",
      location: "Seoul",
      ownershipType: "business",
      profileVisibility: "public",
      publicEmail: "hello@example.com",
      publicMembersVisible: true,
      websiteUrl: "https://acme.example",
    },
    socialAccounts: [
      { position: 1, provider: "x", value: "@acmelabs" },
      {
        position: 2,
        provider: "mastodon",
        value: "https://mastodon.social/@acme",
      },
    ],
    viewerState: {
      canArchive: false,
      canDelete: false,
      canEditProfile: true,
      canRename: true,
      role: "owner",
    },
  };
  return { ...base, ...overrides };
}

function updatedProfileSettings(
  patch: Partial<OrganizationProfileSettings["profile"]>,
): OrganizationProfileSettings {
  const current = profileSettings();
  return {
    ...current,
    profile: {
      ...current.profile,
      ...patch,
    },
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("OrganizationSettingsShell", () => {
  it("renders organization context, grouped navigation, and concrete context-switcher links", () => {
    const settings = profileSettings();
    const { container } = render(
      <OrganizationSettingsShell
        activeSection="profile"
        session={session()}
        settings={settings}
        shellContext={shellContext()}
        title="Profile"
      >
        <p>Settings body</p>
      </OrganizationSettingsShell>,
    );

    expect(screen.getByRole("heading", { name: "Profile" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Acme Labs" })).toBeVisible();
    expect(screen.getByRole("link", { name: "@acme-labs" })).toHaveAttribute(
      "href",
      "/orgs/acme-labs",
    );
    expect(
      screen.getByRole("link", { name: "Personal settings" }),
    ).toHaveAttribute("href", "/settings/profile");
    expect(
      screen.getByRole("link", { name: "Organization settings" }),
    ).toHaveAttribute("href", "/organizations/acme-labs/settings/profile");
    expect(
      screen.getByRole("navigation", {
        name: "Organization settings navigation",
      }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Profile" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByText("Billing")).toHaveAttribute(
      "aria-disabled",
      "true",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
    expect(container.innerHTML).toContain("var(--surface-2)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });
});

describe("OrganizationProfileSettingsForm", () => {
  it("renders initial profile, contact, social, disabled billing, and danger affordances", () => {
    const { container } = render(
      <OrganizationProfileSettingsForm settings={profileSettings()} />,
    );

    expect(screen.getByLabelText("Organization display name")).toHaveValue(
      "Acme Labs",
    );
    expect(screen.getByLabelText("Description")).toHaveValue(
      "Builds quieter developer tools.",
    );
    expect(screen.getByLabelText("URL")).toHaveValue("https://acme.example");
    expect(screen.getByLabelText("Location")).toHaveValue("Seoul");
    expect(screen.getByLabelText("Public email")).toHaveValue(
      "hello@example.com",
    );
    expect(screen.getByLabelText("Contact email")).toHaveValue(
      "admin@example.com",
    );
    expect(screen.getByLabelText("Billing email")).toHaveValue(
      "billing@example.com",
    );
    expect(screen.getByLabelText("X")).toHaveValue("@acmelabs");
    expect(screen.getByLabelText("Mastodon")).toHaveValue(
      "https://mastodon.social/@acme",
    );
    expect(
      screen.getByRole("button", { name: "Upload unavailable" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Save profile changes" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Danger actions unavailable" }),
    ).toBeDisabled();
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });

  it("saves public profile changes through the organization actions route", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      json: async () =>
        updatedProfileSettings({
          description: "Maintainer tools with calmer defaults.",
          displayName: "Acme Research",
          websiteUrl: "https://research.example",
        }),
      ok: true,
    });
    vi.stubGlobal("fetch", fetchMock);
    render(<OrganizationProfileSettingsForm settings={profileSettings()} />);

    fireEvent.change(screen.getByLabelText("Organization display name"), {
      target: { value: "Acme Research" },
    });
    fireEvent.change(screen.getByLabelText("Description"), {
      target: { value: "Maintainer tools with calmer defaults." },
    });
    fireEvent.change(screen.getByLabelText("URL"), {
      target: { value: "https://research.example" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save profile changes" }),
    );

    await waitFor(() =>
      expect(screen.getByText("Public profile updated")).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/organizations/acme-labs/settings/profile/actions",
      expect.objectContaining({
        body: JSON.stringify({
          companyName: "Acme Inc.",
          description: "Maintainer tools with calmer defaults.",
          displayName: "Acme Research",
          location: "Seoul",
          publicEmail: "hello@example.com",
          websiteUrl: "https://research.example",
        }),
        method: "PATCH",
      }),
    );
    expect(
      screen.getByRole("button", { name: "Save profile changes" }),
    ).toBeDisabled();
  });

  it("saves contact and social sections independently and shows server errors without local persistence", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () =>
          updatedProfileSettings({
            billingEmail: "finance@example.com",
            contactEmail: "ops@example.com",
          }),
        ok: true,
      })
      .mockResolvedValueOnce({
        json: async () => ({
          error: {
            code: "validation_failed",
            message: "Unsupported social provider.",
          },
          status: 422,
        }),
        ok: false,
      });
    vi.stubGlobal("fetch", fetchMock);
    render(<OrganizationProfileSettingsForm settings={profileSettings()} />);

    fireEvent.change(screen.getByLabelText("Contact email"), {
      target: { value: "ops@example.com" },
    });
    fireEvent.change(screen.getByLabelText("Billing email"), {
      target: { value: "finance@example.com" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save contact changes" }),
    );
    await waitFor(() =>
      expect(screen.getByText("Administrative contact updated")).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/organizations/acme-labs/settings/profile/actions",
      expect.objectContaining({
        body: JSON.stringify({
          billingEmail: "finance@example.com",
          contactEmail: "ops@example.com",
        }),
        method: "PATCH",
      }),
    );

    fireEvent.change(screen.getByLabelText("X"), {
      target: { value: "@broken" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save social accounts" }),
    );
    await waitFor(() =>
      expect(screen.getByText("Unsupported social provider.")).toBeVisible(),
    );
    expect(screen.getByLabelText("X")).toHaveValue("@broken");
  });

  it("validates obvious bad values before calling the API", () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    render(<OrganizationProfileSettingsForm settings={profileSettings()} />);

    fireEvent.change(screen.getByLabelText("Organization display name"), {
      target: { value: " " },
    });
    fireEvent.change(screen.getByLabelText("URL"), {
      target: { value: "javascript:alert(1)" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save profile changes" }),
    );

    expect(
      screen.getByText("Organization display name is required."),
    ).toBeVisible();
    expect(
      screen.getByText("URL must start with http:// or https://."),
    ).toBeVisible();
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
