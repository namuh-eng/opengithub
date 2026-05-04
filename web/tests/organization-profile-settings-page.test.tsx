import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
});
