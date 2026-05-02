import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizationPeoplePage } from "@/components/OrganizationPeoplePage";
import type {
  OrganizationPeopleList,
  OrganizationPeopleListItem,
} from "@/lib/api";

function person(
  overrides: Partial<OrganizationPeopleListItem> = {},
): OrganizationPeopleListItem {
  return {
    id: "person-1",
    login: "ashley",
    name: "Ashley Ha",
    avatarUrl: null,
    href: "/ashley",
    role: null,
    joinedAt: "2026-04-01T00:00:00Z",
    ...overrides,
  };
}

function peopleList(
  overrides: Partial<OrganizationPeopleList> = {},
): OrganizationPeopleList {
  const items = overrides.items ?? [person()];
  return {
    items,
    total: overrides.total ?? items.length,
    page: overrides.page ?? 1,
    pageSize: overrides.pageSize ?? 30,
    mode: "people",
    filters: {
      query: null,
      page: 1,
      pageSize: 30,
      ...overrides.filters,
    },
    tabCounts: {
      repositories: 4,
      projects: 0,
      packages: 0,
      people: 1,
      sponsoring: 0,
    },
    viewerState: {
      authenticated: false,
      isMember: false,
      role: null,
      canViewInternal: false,
      canAdmin: false,
      isFollowing: false,
    },
    ...overrides,
  };
}

describe("OrganizationPeoplePage", () => {
  it("renders public member rows with concrete profile links and no role leakage", () => {
    const { container } = render(
      <OrganizationPeoplePage
        list={peopleList({
          items: [
            person({
              avatarUrl: "https://images.opengithub.local/ashley.png",
            }),
          ],
        })}
        org="namuh"
      />,
    );

    expect(screen.getByRole("heading", { name: "People" })).toBeVisible();
    expect(screen.getByText("1-1 of 1")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Open Ashley Ha" }),
    ).toHaveAttribute("href", "/ashley");
    expect(container.querySelector(".av.lg")).toHaveStyle({
      backgroundImage: 'url("https://images.opengithub.local/ashley.png")',
    });
    expect(screen.getByText("@ashley")).toBeVisible();
    expect(screen.queryByText("Owner")).toBeNull();
    expect(screen.queryByText("Admin")).toBeNull();
    expect(screen.queryByText("Member")).toBeNull();
    expect(
      screen.getByText(
        "Signed-out and outside viewers see public members only.",
      ),
    ).toBeVisible();
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("renders member-visible roles, permissions nav, and URL-backed filters", () => {
    render(
      <OrganizationPeoplePage
        list={peopleList({
          items: [
            person({ id: "owner", login: "ashley", role: "owner" }),
            person({
              id: "member",
              login: "jaeyun",
              name: "Jaeyun Ha",
              role: "member",
            }),
          ],
          total: 2,
          filters: { query: "jae", page: 1, pageSize: 30 },
          viewerState: {
            authenticated: true,
            isMember: true,
            role: "owner",
            canViewInternal: true,
            canAdmin: true,
            isFollowing: false,
          },
        })}
        org="namuh"
      />,
    );

    expect(screen.getByLabelText("Search organization people")).toHaveValue(
      "jae",
    );
    expect(screen.getByRole("button", { name: "Filter" })).toHaveAttribute(
      "type",
      "submit",
    );
    expect(screen.getByText("Owner")).toBeVisible();
    expect(screen.getByText("Member")).toBeVisible();
    expect(
      screen.getByText("Membership roles are visible to organization members."),
    ).toBeVisible();
    const permissions = screen.getByRole("complementary", {
      name: "Organization permissions",
    });
    expect(
      within(permissions).getByRole("link", { name: "Members" }),
    ).toHaveAttribute("href", "/orgs/namuh/people");
    expect(
      within(permissions).getByRole("link", { name: "Repositories" }),
    ).toHaveAttribute("href", "/orgs/namuh/repositories");
    expect(
      within(permissions).getByRole("link", { name: "Teams" }),
    ).toHaveAttribute("href", "/orgs/namuh/teams/core");
    expect(
      within(permissions).getByRole("link", { name: "Settings" }),
    ).toHaveAttribute("href", "/orgs/namuh/settings");
    expect(screen.getByRole("link", { name: "Search: jae x" })).toHaveAttribute(
      "href",
      "/orgs/namuh/people",
    );
  });

  it("renders pagination and empty-state recovery that preserve search", () => {
    render(
      <OrganizationPeoplePage
        list={peopleList({
          items: [],
          total: 45,
          page: 2,
          pageSize: 10,
          filters: { query: "ash", page: 2, pageSize: 10 },
        })}
        org="namuh"
      />,
    );

    expect(
      screen.getByText("No visible members matched these filters."),
    ).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Clear filters" })[0],
    ).toHaveAttribute("href", "/orgs/namuh/people");
    const pagination = screen.getByRole("navigation", {
      name: "People pagination",
    });
    expect(
      within(pagination).getByRole("link", { name: "Previous" }),
    ).toHaveAttribute("href", "/orgs/namuh/people?q=ash&pageSize=10");
    expect(
      within(pagination).getByRole("link", { name: "Next" }),
    ).toHaveAttribute("href", "/orgs/namuh/people?q=ash&page=3&pageSize=10");
  });

  it("uses disabled real buttons at people pagination boundaries", () => {
    render(
      <OrganizationPeoplePage
        list={peopleList({ items: [person()], total: 1 })}
        org="namuh"
      />,
    );

    const pagination = screen.getByRole("navigation", {
      name: "People pagination",
    });
    expect(
      within(pagination).getByRole("button", { name: "Previous" }),
    ).toBeDisabled();
    expect(
      within(pagination).getByRole("button", { name: "Next" }),
    ).toBeDisabled();
  });
});
