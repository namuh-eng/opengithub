import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SocialListPage } from "@/components/SocialListPage";
import type {
  AuthSession,
  ProfileSocialList,
  RepositoryStargazerList,
} from "@/lib/api";

const session: AuthSession = { authenticated: true, user: null };

describe("SocialListPage", () => {
  it("renders follower rows with concrete profile links", () => {
    const list: ProfileSocialList = {
      items: [
        {
          id: "user-1",
          login: "mona",
          name: "Mona",
          avatarUrl: null,
          bio: "Builds release tools.",
          href: "/mona",
          followedAt: "2026-05-01T00:00:00Z",
          viewerState: {
            authenticated: true,
            isSelf: false,
            isFollowing: false,
            isBlocking: false,
            canFollow: true,
            canBlock: true,
            canReport: true,
          },
        },
      ],
      total: 1,
      page: 1,
      pageSize: 30,
      owner: { login: "ashley", name: "Ashley", href: "/ashley" },
      mode: "followers",
    };

    render(
      <SocialListPage
        backHref="/ashley"
        backLabel="Back to profile"
        empty="No followers"
        eyebrow="Profile social graph"
        list={list}
        session={session}
        title="ashley followers"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "ashley followers" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "mona" })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getByText("Builds release tools.")).toBeVisible();
  });

  it("renders repository stargazers with starred dates", () => {
    const list: RepositoryStargazerList = {
      items: [
        {
          id: "user-2",
          login: "octo",
          name: null,
          avatarUrl: null,
          bio: null,
          href: "/octo",
          starredAt: "2026-05-02T00:00:00Z",
        },
      ],
      total: 1,
      page: 1,
      pageSize: 30,
      repository: {
        ownerLogin: "mona",
        name: "octo-app",
        href: "/mona/octo-app",
      },
    };

    render(
      <SocialListPage
        backHref="/mona/octo-app"
        backLabel="Back to repository"
        empty="No stars"
        eyebrow="Repository stars"
        list={list}
        session={session}
        title="mona/octo-app stargazers"
      />,
    );

    expect(screen.getByRole("link", { name: "octo" })).toHaveAttribute(
      "href",
      "/octo",
    );
    expect(screen.getByText(/Starred May 2, 2026/)).toBeVisible();
  });
});
