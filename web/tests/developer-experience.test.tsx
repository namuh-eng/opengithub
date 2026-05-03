import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "@/components/AppShell";
import { DeveloperTokensPage } from "@/components/DeveloperTokensPage";
import { PersonalAccessTokenCreatePage } from "@/components/PersonalAccessTokenCreatePage";
import type {
  PersonalAccessTokenListFetchResult,
  PersonalAccessTokenNewContextFetchResult,
} from "@/lib/api";

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

const tokenCreateContext: PersonalAccessTokenNewContextFetchResult = {
  ok: true,
  context: {
    sudo: {
      active: true,
      expiresAt: "2026-05-04T12:30:00Z",
      requiredFor: ["create_personal_access_token"],
    },
    resourceOwners: [
      {
        id: "owner-1",
        kind: "user",
        login: "mona",
        displayName: "Mona Lisa",
        avatarUrl: null,
      },
      {
        id: "owner-2",
        kind: "organization",
        login: "namuh",
        displayName: "Namuh",
        avatarUrl: null,
      },
    ],
    repositories: [
      {
        id: "repo-1",
        owner: "mona",
        name: "octo-app",
        fullName: "mona/octo-app",
        visibility: "private",
      },
      {
        id: "repo-2",
        owner: "namuh",
        name: "opengithub",
        fullName: "namuh/opengithub",
        visibility: "private",
      },
    ],
    permissionGroups: [
      {
        key: "repositories",
        label: "Repositories",
        permissions: [
          {
            key: "contents",
            label: "Contents",
            levels: ["none", "read", "write"],
          },
          {
            key: "issues",
            label: "Issues",
            levels: ["none", "read", "write"],
          },
        ],
      },
      {
        key: "packages",
        label: "Packages",
        permissions: [
          {
            key: "packages",
            label: "Packages",
            levels: ["none", "read", "write"],
          },
        ],
      },
      {
        key: "account",
        label: "Account",
        permissions: [
          {
            key: "api",
            label: "REST API",
            levels: ["none", "read", "write"],
          },
        ],
      },
    ],
    defaultExpirationDays: 30,
    maxExpirationDays: 366,
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

  it("confirms token revoke before forwarding the delete request", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        revokedAt: "2026-05-04T01:00:00Z",
        token: {
          ...populatedTokenList.list.tokens[0],
          status: "revoked",
          revokedAt: "2026-05-04T01:00:00Z",
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<DeveloperTokensPage tokenList={populatedTokenList} />);

    fireEvent.click(screen.getByRole("button", { name: "Revoke" }));
    expect(screen.getByText("Revoke Deploy token")).toBeVisible();
    expect(screen.getByRole("button", { name: "Revoke token" })).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Confirm revoke Deploy token"), {
      target: { value: "Deploy token" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Revoke token" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/personal-access-tokens/token-1",
      { method: "DELETE" },
    );
    expect(await screen.findByText("Deploy token revoked.")).toBeVisible();
    expect(screen.getByText("Revoked")).toBeVisible();
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

  it("renders fine-grained token create controls from server context and prefill", () => {
    const { container } = render(
      <PersonalAccessTokenCreatePage
        contextResult={tokenCreateContext}
        initialQuery={{
          contents: "write",
          description: "From query",
          name: "Deploy from query",
          packages: "read",
          target_name: "namuh",
        }}
        userEmail="mona@example.com"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "New fine-grained token" }),
    ).toBeVisible();
    expect(screen.getByDisplayValue("Deploy from query")).toBeVisible();
    expect(screen.getByDisplayValue("From query")).toBeVisible();
    expect(screen.getByDisplayValue("namuh (organization)")).toBeVisible();
    expect(screen.getByText("namuh/opengithub")).toBeVisible();
    expect(screen.getByDisplayValue("write")).toBeVisible();
    expect(screen.getAllByDisplayValue("read").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("button", { name: "Generate token" }),
    ).toBeEnabled();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders classic token mode and submits broad-scope creation", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        createdAt: "2026-05-04T00:00:00Z",
        plainTextToken: "oghp_classic_secret",
        token: {
          ...populatedTokenList.list.tokens[0],
          id: "classic-token",
          name: "Classic browser token",
          prefix: "oghp_classic_sec",
          type: "classic",
          repositoryAccess: "all",
          scopes: ["repo", "api:read"],
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <PersonalAccessTokenCreatePage
        contextResult={tokenCreateContext}
        initialQuery={{ type: "classic", name: "Classic browser token" }}
        userEmail="mona@example.com"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "New classic token" }),
    ).toBeVisible();
    expect(screen.getByText(/Classic tokens use broad access/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Generate token" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/personal-access-tokens/actions",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining('"type":"classic"'),
      }),
    );
    expect(await screen.findByText("oghp_classic_secret")).toBeVisible();
  });

  it("submits fine-grained token creation and reveals plaintext once", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        createdAt: "2026-05-04T00:00:00Z",
        plainTextToken: "oghp_generated_secret",
        token: {
          ...populatedTokenList.list.tokens[0],
          id: "created-token",
          name: "Browser token",
          prefix: "oghp_generated_s",
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    });

    render(
      <PersonalAccessTokenCreatePage
        contextResult={tokenCreateContext}
        userEmail="mona@example.com"
      />,
    );

    fireEvent.change(screen.getByLabelText("Token name"), {
      target: { value: "Browser token" },
    });
    fireEvent.click(screen.getByText("mona/octo-app"));
    fireEvent.click(screen.getByRole("button", { name: "Generate token" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/personal-access-tokens/actions",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining("Browser token"),
      }),
    );
    expect(await screen.findByText("oghp_generated_secret")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Copy token" }));
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
      "oghp_generated_secret",
    );
    expect(await screen.findByText("Copied")).toBeVisible();
  });
});
