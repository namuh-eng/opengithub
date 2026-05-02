import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { NotificationsInboxPage } from "@/components/NotificationsInboxPage";
import type { NotificationInboxView } from "@/lib/api";

function inboxView(
  overrides: Partial<NotificationInboxView> = {},
): NotificationInboxView {
  return {
    query: {
      q: "reason:mention",
      folder: "inbox",
      tab: "all",
      sort: "newest",
      group: "repository",
      repo: null,
    },
    folders: [
      {
        id: "inbox",
        label: "Inbox",
        query: "",
        href: "/notifications",
        count: 2,
        active: true,
      },
      {
        id: "saved",
        label: "Saved",
        query: "",
        href: "/notifications?folder=saved",
        count: 0,
        active: false,
      },
      {
        id: "done",
        label: "Done",
        query: "",
        href: "/notifications?folder=done",
        count: 0,
        active: false,
      },
    ],
    filters: [
      {
        id: "assigned",
        label: "Assigned",
        query: "reason:assigned",
        href: "/notifications?q=reason%3Aassigned",
        count: 1,
        active: false,
      },
      {
        id: "mentioned",
        label: "Mentioned",
        query: "reason:mention",
        href: "/notifications?q=reason%3Amention",
        count: 1,
        active: true,
      },
      {
        id: "review-requested",
        label: "Review requested",
        query: "reason:review_requested",
        href: "/notifications?q=reason%3Areview_requested",
        count: 0,
        active: false,
      },
    ],
    repositories: [
      {
        id: "mona/octo-app",
        label: "mona/octo-app",
        query: "",
        href: "/notifications?repo=mona%2Focto-app",
        count: 2,
        active: false,
      },
    ],
    sortOptions: [
      {
        id: "newest",
        label: "Newest",
        href: "/notifications?q=reason%3Amention&group=repository",
        active: true,
      },
      {
        id: "oldest",
        label: "Oldest",
        href: "/notifications?q=reason%3Amention&sort=oldest&group=repository",
        active: false,
      },
    ],
    groupOptions: [
      {
        id: "date",
        label: "Date",
        href: "/notifications?q=reason%3Amention",
        active: false,
      },
      {
        id: "repository",
        label: "Repository",
        href: "/notifications?q=reason%3Amention&group=repository",
        active: true,
      },
    ],
    groups: [
      {
        id: "mona/octo-app",
        label: "mona/octo-app",
        count: 1,
        rows: [
          {
            id: "notif-1",
            repositoryId: "repo-1",
            repositoryName: "mona/octo-app",
            repositoryHref: "/mona/octo-app",
            subjectType: "issue",
            subjectNumber: 42,
            title: "Inbox search keeps mention filters",
            reason: "mention",
            reasonLabel: "Mention",
            href: "/mona/octo-app/issues/42",
            openHref:
              "/notifications/notif-1/open?next=%2Fmona%2Focto-app%2Fissues%2F42",
            unread: true,
            saved: false,
            done: false,
            subscribed: true,
            updatedAt: "2026-05-02T00:00:00Z",
            relativeTime: "2h ago",
          },
        ],
      },
    ],
    total: 1,
    unreadCount: 1,
    page: 1,
    pageSize: 50,
    emptyTitle: "No matching notifications",
    emptyMessage: "Adjust filters.",
    ...overrides,
  };
}

describe("NotificationsInboxPage", () => {
  it("renders folders, filters, repository buckets, controls, and notification row states", () => {
    render(<NotificationsInboxPage view={inboxView()} />);

    expect(
      screen.getByRole("heading", { name: "1 notifications" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Manage notifications/ }),
    ).toHaveAttribute("href", "/settings/notifications");
    expect(screen.getByRole("link", { name: /Saved/ })).toHaveAttribute(
      "href",
      "/notifications?folder=saved",
    );
    expect(screen.getByRole("link", { name: /Mentioned/ })).toHaveAttribute(
      "href",
      "/notifications?q=reason%3Amention",
    );
    expect(
      within(
        screen.getByRole("navigation", {
          name: "Repository notification buckets",
        }),
      ).getByRole("link", { name: /mona\/octo-app/ }),
    ).toHaveAttribute("href", "/notifications?repo=mona%2Focto-app");

    const search = screen.getByRole("searchbox", {
      name: "Search notifications",
    });
    expect(search).toHaveAttribute("name", "q");
    expect(search).toHaveValue("reason:mention");
    expect(screen.getByRole("link", { name: "Unread" })).toHaveAttribute(
      "href",
      "/notifications?tab=unread&q=reason%3Amention&group=repository",
    );
    expect(screen.getByRole("link", { name: "Oldest" })).toHaveAttribute(
      "href",
      "/notifications?q=reason%3Amention&sort=oldest&group=repository",
    );

    const group = screen.getByRole("region", { name: "mona/octo-app" });
    expect(
      within(group).getByRole("link", {
        name: /Inbox search keeps mention filters/,
      }),
    ).toHaveAttribute(
      "href",
      "/notifications/notif-1/open?next=%2Fmona%2Focto-app%2Fissues%2F42",
    );
    expect(within(group).getByText("Mention")).toBeVisible();
    expect(within(group).getByText("Subscribed")).toBeVisible();
    expect(within(group).getByLabelText("Unread")).toBeVisible();
  });

  it("keeps filters editable in an empty state", () => {
    render(
      <NotificationsInboxPage
        view={inboxView({
          groups: [],
          total: 0,
          emptyTitle: "No unread notifications",
        })}
      />,
    );

    expect(screen.getByText("No unread notifications")).toBeVisible();
    expect(
      screen.getByRole("searchbox", { name: "Search notifications" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "All" })).toHaveAttribute(
      "href",
      "/notifications?q=reason%3Amention&group=repository",
    );
  });
});
