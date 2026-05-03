import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "@/components/AppShell";
import { DeveloperTokensPage } from "@/components/DeveloperTokensPage";
import type { PersonalAccessTokenListFetchResult } from "@/lib/api";

const emptyTokenList: PersonalAccessTokenListFetchResult = {
  ok: true,
  list: {
    sudo: {
      active: false,
      expiresAt: null,
      requiredFor: [
        "create_personal_access_token",
        "revoke_personal_access_token",
      ],
    },
    tokens: [],
  },
};

const populatedTokenList: PersonalAccessTokenListFetchResult = {
  ok: true,
  list: {
    sudo: {
      active: true,
      expiresAt: "2026-05-04T12:30:00Z",
      requiredFor: [
        "create_personal_access_token",
        "revoke_personal_access_token",
      ],
    },
    tokens: [
      {
        id: "token-1",
        name: "Deploy token",
        description: "Used by release automation",
        type: "fine_grained",
        prefix: "oghp_12345678",
        scopes: ["repo:read", "packages:write"],
        resourceOwner: {
          id: "owner-1",
          kind: "organization",
          login: "namuh",
          displayName: "Namuh",
          avatarUrl: null,
        },
        repositoryAccess: "selected",
        selectedRepositories: [
          {
            id: "repo-1",
            owner: "namuh",
            name: "opengithub",
            fullName: "namuh/opengithub",
            visibility: "private",
          },
        ],
        status: "active",
        lastUsedAt: null,
        expiresAt: "2026-06-04T00:00:00Z",
        revokedAt: null,
        createdAt: "2026-05-04T00:00:00Z",
      },
    ],
  },
};

describe("DeveloperTokensPage", () => {
  it("renders opengithub token workflow docs without placeholder controls", () => {
    const { container } = render(
      <DeveloperTokensPage tokenList={emptyTokenList} />,
    );

    expect(
      screen.getByRole("heading", { name: "Personal access tokens" }),
    ).toBeVisible();
    expect(screen.getByText("No personal access tokens yet")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "New fine-grained token" }),
    ).toHaveAttribute(
      "href",
      "/settings/personal-access-tokens/new?type=fine_grained",
    );
    expect(
      screen.getByRole("link", { name: "New classic token" }),
    ).toHaveAttribute(
      "href",
      "/settings/personal-access-tokens/new?type=classic",
    );
    expect(screen.getByText("Token quickstart")).toBeVisible();
    expect(screen.getByText("repo:read")).toBeVisible();
    expect(screen.getByText("repo:write")).toBeVisible();
    expect(screen.getByText("api:read")).toBeVisible();
    expect(screen.getByText("api:write")).toBeVisible();
    expect(
      screen.getByText((content) =>
        content.includes("https://opengithub.namuh.co/api/user"),
      ),
    ).toBeVisible();
    expect(container).not.toHaveTextContent("api.github.com");
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders server-backed token rows without secret material", () => {
    const { container } = render(
      <DeveloperTokensPage tokenList={populatedTokenList} />,
    );

    expect(screen.getByText("Deploy token")).toBeVisible();
    expect(screen.getByText("Used by release automation")).toBeVisible();
    expect(screen.getByText("oghp_12345678")).toBeVisible();
    expect(screen.getByText("namuh")).toBeVisible();
    expect(screen.getByText("namuh/opengithub")).toBeVisible();
    expect(
      screen.getByText("Sudo mode is active for this session."),
    ).toBeVisible();
    expect(container).not.toHaveTextContent("sha256:");
    expect(container).not.toHaveTextContent("oghp_actual_secret");
  });

  it("renders unavailable and unauthorized states with concrete sign-in link", () => {
    const { container } = render(
      <DeveloperTokensPage
        tokenList={{
          ok: false,
          status: 401,
          code: "unauthorized",
          message: "Authentication required",
        }}
      />,
    );

    expect(
      screen.getByText("Token settings could not be loaded."),
    ).toBeVisible();
    expect(
      screen.getByText("Sign in to manage personal access tokens."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      "/login?next=/settings/tokens",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("copies token command snippets", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<DeveloperTokensPage tokenList={emptyTokenList} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy API curl" }));

    expect(writeText).toHaveBeenCalledWith(
      expect.stringContaining("https://opengithub.namuh.co/api/user"),
    );
    expect(await screen.findByRole("status")).toHaveTextContent("Copied");
  });

  it("links signed-in users to developer settings from the avatar menu", () => {
    render(
      <AppShell
        session={{
          authenticated: true,
          user: {
            id: "user-1",
            email: "mona@example.com",
            display_name: "Mona Lisa",
            avatar_url: null,
          },
        }}
      >
        <DeveloperTokensPage tokenList={emptyTokenList} />
      </AppShell>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Open user menu" }));
    expect(
      screen.getByRole("menuitem", { name: "Developer settings" }),
    ).toHaveAttribute("href", "/settings/tokens");
  });
});
