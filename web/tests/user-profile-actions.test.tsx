import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UserProfileActions } from "@/components/UserProfileActions";
import type { ProfileViewerState } from "@/lib/api";

function viewerState(
  overrides: Partial<ProfileViewerState> = {},
): ProfileViewerState {
  return {
    authenticated: true,
    canBlock: true,
    canFollow: true,
    canReport: true,
    isBlocking: false,
    isFollowing: false,
    isSelf: false,
    ...overrides,
  };
}

function mockFetch(response: unknown, ok = true) {
  return vi.fn().mockResolvedValue({
    json: async () => response,
    ok,
  }) as unknown as typeof fetch;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("UserProfileActions", () => {
  it("optimistically follows and reconciles the returned follower count", async () => {
    global.fetch = mockFetch({
      followerCount: 43,
      viewerState: viewerState({ isFollowing: true }),
    });

    render(
      <UserProfileActions
        followerCount={42}
        followingCount={18}
        isPrivate={false}
        login="ashley"
        viewerState={viewerState()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Follow" }));
    expect(screen.getByText("43 followers · 18 following")).toBeVisible();

    await waitFor(() =>
      expect(global.fetch).toHaveBeenCalledWith("/ashley/actions/follow", {
        method: "PUT",
      }),
    );
    expect(screen.getByRole("button", { name: "Following" })).toBeVisible();
    expect(screen.getByText("Now following this profile.")).toBeVisible();
  });

  it("rolls back a failed follow update and shows the API error", async () => {
    global.fetch = mockFetch(
      {
        error: {
          code: "validation_failed",
          message: "profile action cannot target your own account",
        },
        status: 422,
      },
      false,
    );

    render(
      <UserProfileActions
        followerCount={2}
        followingCount={1}
        isPrivate={false}
        login="ashley"
        viewerState={viewerState()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Follow" }));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Follow" })).toBeVisible(),
    );
    expect(screen.getByText("2 followers · 1 following")).toBeVisible();
    expect(
      screen.getByText("profile action cannot target your own account"),
    ).toBeVisible();
  });

  it("opens a login gate instead of posting anonymous actions", () => {
    global.fetch = vi.fn() as unknown as typeof fetch;

    render(
      <UserProfileActions
        followerCount={2}
        followingCount={1}
        isPrivate={false}
        login="ashley"
        viewerState={viewerState({ authenticated: false })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Follow" }));

    expect(
      screen.getByRole("dialog", { name: "Continue to interact with @ashley" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      expect.stringContaining("/login?next="),
    );
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it("submits block and report through concrete menu dialogs", async () => {
    global.fetch = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () => ({
          followerCount: 0,
          viewerState: viewerState({ isBlocking: true }),
        }),
        ok: true,
      })
      .mockResolvedValueOnce({
        json: async () => ({
          id: "report-1",
          viewerState: viewerState({ isBlocking: true }),
        }),
        ok: true,
      }) as unknown as typeof fetch;

    render(
      <UserProfileActions
        followerCount={1}
        followingCount={0}
        isPrivate={false}
        login="ashley"
        viewerState={viewerState()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "More" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Block profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Block" }));

    await waitFor(() =>
      expect(global.fetch).toHaveBeenCalledWith("/ashley/actions/block", {
        body: JSON.stringify({ reason: "User requested block from profile" }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      }),
    );
    expect(screen.getByText("Profile blocked.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "More" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Report profile" }));
    const dialog = screen.getByRole("dialog", {
      name: "Tell us what is wrong with @ashley",
    });
    fireEvent.change(within(dialog).getByLabelText("Details"), {
      target: { value: "This profile is sending spam." },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Submit report" }),
    );

    await waitFor(() =>
      expect(global.fetch).toHaveBeenLastCalledWith("/ashley/actions/report", {
        body: JSON.stringify({
          details: "This profile is sending spam.",
          reason: "spam",
        }),
        headers: { "content-type": "application/json" },
        method: "POST",
      }),
    );
    expect(screen.getByText("Report submitted for review.")).toBeVisible();
  });

  it("hides meaningless controls for private and self profile states", () => {
    const { rerender } = render(
      <UserProfileActions
        followerCount={null}
        followingCount={null}
        isPrivate={true}
        login="ashley"
        viewerState={viewerState()}
      />,
    );

    expect(screen.queryByRole("button", { name: "Follow" })).toBeNull();
    expect(screen.queryByRole("button", { name: "More" })).toBeNull();

    rerender(
      <UserProfileActions
        followerCount={4}
        followingCount={2}
        isPrivate={false}
        login="ashley"
        viewerState={viewerState({ isSelf: true })}
      />,
    );
    expect(screen.getByText("Your profile")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Follow" })).toBeNull();
  });
});
