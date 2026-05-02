import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { GlobalSearchModal } from "@/components/GlobalSearchModal";

function suggestionDashboard() {
  return {
    groups: [
      {
        id: "scopes",
        title: "Search scopes",
        items: [
          {
            id: "scope-repositories",
            kind: "submit_search",
            action: "submit_search",
            title: "Repositories",
            description: "Search repository names and descriptions",
            href: "/search?q=router&type=repositories",
            nextQuery: "router",
            scope: "repositories",
            ownerLogin: null,
            repositoryName: null,
            visibility: null,
          },
        ],
      },
      {
        id: "qualifiers",
        title: "Query qualifiers",
        items: [
          {
            id: "qualifier-language",
            kind: "replace_token",
            action: "replace_token",
            title: "language:rust",
            description: "Limit code results by language",
            href: null,
            nextQuery: "language:rust ",
            scope: null,
            ownerLogin: null,
            repositoryName: null,
            visibility: null,
          },
        ],
      },
      {
        id: "repositories",
        title: "Repositories and code",
        items: [
          {
            id: "repo-1",
            kind: "direct_repository_jump",
            action: "navigate",
            title: "mona/editorial",
            description: "public repository",
            href: "/mona/editorial",
            nextQuery: null,
            scope: null,
            ownerLogin: "mona",
            repositoryName: "editorial",
            visibility: "public",
          },
        ],
      },
    ],
    query: "router",
    recentSearches: [
      {
        id: "recent-1",
        query: "router guards",
        scope: "all",
        resultType: "repositories",
        href: "/search?q=router+guards&type=repositories",
        searchedAt: "2026-05-02T00:00:00Z",
      },
    ],
    savedSearches: [
      {
        id: "saved-1",
        name: "Rust files",
        query: "language:rust",
        scope: "code",
        href: "/search?q=language%3Arust&type=code",
        updatedAt: "2026-05-02T00:00:00Z",
      },
    ],
    scope: "all",
    token: {
      prefix: "language",
      value: "language:ru",
      replaceFrom: 0,
      replaceTo: 11,
    },
  };
}

beforeEach(() => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = input.toString();
      if (url === "/search/saved-searches" && init?.method === "POST") {
        const body = JSON.parse(String(init.body));
        if (!body.name) {
          return Response.json(
            {
              error: {
                code: "validation_failed",
                message: "saved search name is required",
              },
              status: 422,
            },
            { status: 422 },
          );
        }
        return Response.json({
          id: "saved-2",
          name: body.name,
          query: body.query,
          scope: body.scope,
          href: "/search?q=router&type=repositories",
          updatedAt: "2026-05-02T00:00:00Z",
        });
      }
      if (
        url === "/search/saved-searches/saved-1" &&
        init?.method === "DELETE"
      ) {
        return new Response(null, { status: 204 });
      }
      return Response.json(suggestionDashboard());
    }),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("GlobalSearchModal guardrails", () => {
  it("exposes real modal controls, grouped suggestions, and no inert links", async () => {
    const onClose = vi.fn();
    const { container } = render(
      <GlobalSearchModal initialQuery="router" onClose={onClose} />,
    );

    expect(screen.getByRole("dialog", { name: "Search" })).toHaveAttribute(
      "aria-modal",
      "true",
    );
    expect(
      await screen.findByRole("combobox", { name: "Search opengithub" }),
    ).toHaveFocus();
    expect(screen.getByRole("link", { name: "Syntax tips" })).toHaveAttribute(
      "href",
      "/docs/api#search",
    );
    expect(screen.getByRole("link", { name: "Feedback" })).toHaveAttribute(
      "href",
      "/issues/new?title=Search%20feedback",
    );
    expect(
      await screen.findByRole("option", { name: /mona\/editorial/ }),
    ).toHaveAttribute("href", "/mona/editorial");
    expect(screen.queryByText(/copilot/i)).not.toBeInTheDocument();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("traps focus, closes from Escape on any focused control, and preserves saved-dialog focus return", async () => {
    const onClose = vi.fn();
    render(<GlobalSearchModal initialQuery="language:ru" onClose={onClose} />);

    await waitFor(() =>
      expect(
        screen.getAllByRole("option", { name: /language:rust/ }).length,
      ).toBeGreaterThan(0),
    );
    const manageLink = screen.getByRole("link", {
      name: "Manage saved searches",
    });
    manageLink.focus();
    fireEvent.keyDown(manageLink, { key: "Tab" });
    expect(screen.getByRole("link", { name: "Syntax tips" })).toHaveFocus();

    fireEvent.click(
      screen.getByRole("button", { name: "Create saved search" }),
    );
    expect(
      await screen.findByRole("dialog", { name: "Create saved search" }),
    ).toBeVisible();
    await waitFor(() => expect(screen.getByLabelText("Name")).toHaveFocus());

    fireEvent.keyDown(screen.getByLabelText("Name"), { key: "Escape" });
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Create saved search" }),
      ).not.toBeInTheDocument(),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("combobox", { name: "Search opengithub" }),
      ).toHaveFocus(),
    );

    const feedback = screen.getByRole("link", { name: "Feedback" });
    feedback.focus();
    fireEvent.keyDown(feedback, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("validates saved searches and keeps delete management concrete", async () => {
    render(<GlobalSearchModal initialQuery="router" onClose={vi.fn()} />);

    await screen.findByRole("option", { name: /Rust files/ });
    fireEvent.click(
      screen.getByRole("button", { name: "Create saved search" }),
    );
    fireEvent.click(
      screen.getAllByRole("button", { name: "Create saved search" }).at(-1) ??
        screen.getByRole("button", { name: "Create saved search" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Name is required.",
    );

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Router guardrails" },
    });
    fireEvent.change(screen.getByLabelText("Query"), {
      target: { value: "router language:rust" },
    });
    fireEvent.click(
      screen.getAllByRole("button", { name: "Create saved search" }).at(-1) ??
        screen.getByRole("button", { name: "Create saved search" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      'Saved "Router guardrails".',
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      'Deleted "Rust files".',
    );
  });
});
