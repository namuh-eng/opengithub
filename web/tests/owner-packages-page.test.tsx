import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OwnerPackagesPage } from "@/components/OwnerPackagesPage";
import type { OwnerPackageList } from "@/lib/api";
import { ownerPackagesHref } from "@/lib/navigation";

function packageList(
  overrides: Partial<OwnerPackageList> = {},
): OwnerPackageList {
  const base: OwnerPackageList = {
    items: [
      {
        id: "package-1",
        name: "opengithub-web",
        packageType: "container",
        typeLabel: "Container",
        visibility: "public",
        href: "/ashley/container/opengithub-web",
        publishedAt: "2026-05-01T00:00:00Z",
        publisher: {
          id: "user-1",
          login: "ashley",
          name: "Ashley Ha",
          href: "/ashley",
        },
        linkedRepository: {
          id: "repo-1",
          owner: "ashley",
          name: "opengithub",
          fullName: "ashley/opengithub",
          href: "/ashley/opengithub",
          visibility: "public",
        },
        downloadCount: 1204,
        latestVersion: "1.0.0",
      },
      {
        id: "package-2",
        name: "opengithub-internal",
        packageType: "npm",
        typeLabel: "npm",
        visibility: "internal",
        href: "/ashley/npm/opengithub-internal",
        publishedAt: "2026-04-30T00:00:00Z",
        publisher: {
          id: "user-1",
          login: "ashley",
          name: null,
          href: "/ashley",
        },
        linkedRepository: null,
        downloadCount: 44,
        latestVersion: null,
      },
    ],
    total: 2,
    page: 1,
    pageSize: 30,
    owner: {
      id: "user-1",
      login: "ashley",
      kind: "user",
      href: "/ashley",
    },
    mode: "packages",
    filters: {
      query: null,
      packageType: "all",
      visibility: "all",
      sort: "downloads-desc",
      artifactTab: "packages",
      page: 1,
      pageSize: 30,
    },
    linkedArtifacts: {
      enabled: false,
      message:
        "Linked artifact provenance is not implemented yet; package repository links are shown in the package list.",
    },
  };

  return { ...base, ...overrides };
}

describe("OwnerPackagesPage", () => {
  it("renders filter controls, package metadata, and working links", () => {
    render(
      <OwnerPackagesPage
        list={packageList()}
        owner="ashley"
        ownerKind="user"
      />,
    );

    expect(
      screen.getByRole("link", { name: "GitHub Packages" }),
    ).toHaveAttribute("href", "/ashley?tab=packages");
    expect(
      screen.getByRole("link", { name: "Linked artifacts" }),
    ).toHaveAttribute("href", "/ashley?tab=packages&artifactTab=artifacts");
    expect(screen.getByLabelText("Search packages")).toHaveAttribute(
      "placeholder",
      "Search by package name",
    );
    expect(screen.getByLabelText("Type")).toHaveValue("all");
    expect(screen.getByLabelText("Visibility")).toHaveValue("all");
    expect(screen.getByLabelText("Sort")).toHaveValue("downloads-desc");
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/ashley?tab=packages",
    );

    const packageRow = screen.getByText("opengithub-web").closest("article");
    expect(packageRow).not.toBeNull();
    const row = within(packageRow as HTMLElement);
    expect(row.getByText("Container")).toBeInTheDocument();
    expect(row.getByText(/Published May 1, 2026 by/)).toBeInTheDocument();
    expect(row.getByRole("link", { name: "Ashley Ha" })).toHaveAttribute(
      "href",
      "/ashley",
    );
    expect(
      row.getByRole("link", { name: "ashley/opengithub" }),
    ).toHaveAttribute("href", "/ashley/opengithub");
    expect(row.getByText("1,204 downloads")).toBeInTheDocument();
    expect(screen.getByText("internal")).toBeInTheDocument();
  });

  it("shows filtered empty state with a back-to-all link", () => {
    render(
      <OwnerPackagesPage
        list={packageList({
          items: [],
          total: 0,
          filters: {
            query: "nope",
            packageType: "npm",
            visibility: "private",
            sort: "downloads-asc",
            artifactTab: "packages",
            page: 1,
            pageSize: 30,
          },
        })}
        owner="ashley"
        ownerKind="user"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "No packages matched" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Back to all packages" }),
    ).toHaveAttribute("href", "/ashley?tab=packages");
  });

  it("renders the linked artifacts placeholder tab for org canonical route", () => {
    render(
      <OwnerPackagesPage
        list={packageList({
          owner: {
            id: "org-1",
            login: "namuh",
            kind: "organization",
            href: "/orgs/namuh",
          },
          filters: {
            query: null,
            packageType: "all",
            visibility: "all",
            sort: "downloads-desc",
            artifactTab: "artifacts",
            page: 1,
            pageSize: 30,
          },
        })}
        owner="namuh"
        ownerKind="organization"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Linked artifacts" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "View GitHub Packages" }),
    ).toHaveAttribute("href", "/orgs/namuh/packages");
  });
});

describe("ownerPackagesHref", () => {
  it("serializes user and org package filters without inert URLs", () => {
    expect(
      ownerPackagesHref("user", "ashley", {
        artifactTab: "packages",
        page: 1,
        pageSize: 30,
        query: "api server",
        sort: "downloads-asc",
        type: "npm",
        visibility: "private",
      }),
    ).toBe(
      "/ashley?tab=packages&q=api+server&type=npm&visibility=private&sort=downloads-asc",
    );
    expect(
      ownerPackagesHref(
        "organization",
        "namuh",
        {},
        { artifactTab: "artifacts" },
      ),
    ).toBe("/orgs/namuh/packages?artifactTab=artifacts");
  });
});
