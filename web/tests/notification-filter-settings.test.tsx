import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NotificationFilterSettingsPage } from "@/components/NotificationFilterSettingsPage";
import type { NotificationFilterSettings } from "@/lib/api";

function filterSettings(
  overrides: Partial<NotificationFilterSettings> = {},
): NotificationFilterSettings {
  return {
    defaultFilters: [
      {
        id: "assigned",
        name: "Assigned",
        queryString: "reason:assigned",
        href: "/notifications?q=reason%3Aassigned",
      },
      {
        id: "review-requested",
        name: "Review requested",
        queryString: "reason:review_requested",
        href: "/notifications?q=reason%3Areview_requested",
      },
    ],
    customFilters: [
      {
        id: "filter-1",
        name: "My reviews",
        queryString: "repo:mona/octo-app reason:review_requested",
        position: 1,
        href: "/notifications?q=repo%3Amona%2Focto-app%20reason%3Areview_requested",
        createdAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    limit: 15,
    remaining: 14,
    allowedQualifiers: ["repo", "org", "author", "is", "reason"],
    ...overrides,
  };
}

describe("NotificationFilterSettingsPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders default and custom filter rows with concrete inbox links", () => {
    const { container } = render(
      <NotificationFilterSettingsPage initialSettings={filterSettings()} />,
    );

    expect(screen.getByRole("heading", { name: "Filters" })).toBeVisible();
    expect(screen.getByText("1/15 custom")).toBeVisible();
    expect(screen.getAllByText("Default")).toHaveLength(2);
    expect(screen.getByText("My reviews")).toBeVisible();
    expect(screen.getAllByRole("link", { name: "Open" })[0]).toHaveAttribute(
      "href",
      "/notifications?q=reason%3Aassigned",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("creates a custom filter and surfaces it as a rail link", async () => {
    const fetchMock = vi.fn(
      async () =>
        new Response(
          JSON.stringify(
            filterSettings({
              customFilters: [
                ...filterSettings().customFilters,
                {
                  id: "filter-2",
                  name: "Mentions",
                  queryString: "reason:mention is:unread",
                  position: 2,
                  href: "/notifications?q=reason%3Amention%20is%3Aunread",
                  createdAt: "2026-05-04T01:00:00Z",
                  updatedAt: "2026-05-04T01:00:00Z",
                },
              ],
              remaining: 13,
            }),
          ),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
    );
    vi.stubGlobal("fetch", fetchMock);
    render(
      <NotificationFilterSettingsPage initialSettings={filterSettings()} />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Mentions" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Query" }), {
      target: { value: "reason:mention is:unread" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/notifications/actions",
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            name: "Mentions",
            queryString: "reason:mention is:unread",
          }),
        },
      ),
    );
    expect(await screen.findByText("Filter created.")).toBeVisible();
    const mentionsRow = screen.getByText("Mentions").closest("tr");
    expect(
      within(mentionsRow as HTMLElement).getByRole("link", { name: "Open" }),
    ).toHaveAttribute(
      "href",
      "/notifications?q=reason%3Amention%20is%3Aunread",
    );
  });

  it("blocks full-text and exclusion queries before submit", () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    render(
      <NotificationFilterSettingsPage initialSettings={filterSettings()} />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Bad filter" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Query" }), {
      target: { value: "-repo:mona/octo-app" },
    });

    expect(screen.getByRole("button", { name: "Create" })).toBeDisabled();
    expect(
      screen.getByText(
        "Custom filters do not support NOT or exclusion searches.",
      ),
    ).toBeVisible();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("edits and deletes existing filters through server-confirmed actions", async () => {
    const updated = filterSettings({
      customFilters: [
        {
          ...filterSettings().customFilters[0],
          name: "Updated reviews",
          queryString: "reason:review_requested is:unread",
        },
      ],
    });
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify(updated), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(filterSettings({ customFilters: [] })), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );
    vi.stubGlobal("fetch", fetchMock);
    render(
      <NotificationFilterSettingsPage initialSettings={filterSettings()} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Updated reviews" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Query" }), {
      target: { value: "reason:review_requested is:unread" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save changes" }));

    await screen.findByText("Filter updated.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/settings/notifications/actions",
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          id: "filter-1",
          name: "Updated reviews",
          queryString: "reason:review_requested is:unread",
        }),
      },
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(
      screen.getByRole("dialog", { name: /Remove Updated reviews/ }),
    ).toBeVisible();
    fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Delete",
      }),
    );
    await screen.findByText("Filter deleted.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/settings/notifications/actions",
      {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: "filter-1" }),
      },
    );
  });
});
