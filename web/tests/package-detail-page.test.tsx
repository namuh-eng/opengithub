import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PackageDetailPage } from "@/components/PackageDetailPage";
import type { PackageDetail, PackageDetailFetchResult } from "@/lib/api";

function packageDetail(overrides: Partial<PackageDetail> = {}): PackageDetail {
  const base: PackageDetail = {
    id: "package-1",
    name: "opengithub-web",
    packageType: "container",
    typeLabel: "Container",
    visibility: "public",
    href: "/ashley/container/opengithub-web",
    owner: {
      id: "user-1",
      login: "ashley",
      kind: "user",
      href: "/ashley",
    },
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
    publishedAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-02T00:00:00Z",
    downloadCount: 3620,
    selectedVersion: {
      id: "version-1",
      version: "1.0.0",
      digest: "sha256:abcdef1234567890",
      shortDigest: "sha256:abcdef1",
      platformOs: "linux",
      platformArch: "amd64",
      sizeBytes: 24_576,
      publishedAt: "2026-05-01T00:00:00Z",
      publisher: {
        id: "user-1",
        login: "ashley",
        name: "Ashley Ha",
        href: "/ashley",
      },
      href: "/ashley/container/opengithub-web?version=1.0.0",
    },
    versions: [
      {
        id: "version-1",
        version: "1.0.0",
        digest: "sha256:abcdef1234567890",
        shortDigest: "sha256:abcdef1",
        platformOs: "linux",
        platformArch: "amd64",
        sizeBytes: 24_576,
        publishedAt: "2026-05-01T00:00:00Z",
        publisher: {
          id: "user-1",
          login: "ashley",
          name: "Ashley Ha",
          href: "/ashley",
        },
        href: "/ashley/container/opengithub-web?version=1.0.0",
      },
      {
        id: "version-2",
        version: "0.9.0",
        digest: "sha256:9999999999999999",
        shortDigest: "sha256:9999999",
        platformOs: "darwin",
        platformArch: "arm64",
        sizeBytes: 12_288,
        publishedAt: "2026-04-20T00:00:00Z",
        publisher: {
          id: "user-1",
          login: "ashley",
          name: null,
          href: "/ashley",
        },
        href: "/ashley/container/opengithub-web?version=0.9.0",
      },
    ],
    installCommands: [
      {
        label: "Container",
        command:
          "docker pull ghcr.io/ashley/opengithub-web:1.0.0@sha256:abcdef1234567890",
        version: "1.0.0",
        digest: "sha256:abcdef1234567890",
        platform: "linux/amd64",
      },
    ],
    blobs: [
      {
        id: "blob-1",
        versionId: "version-1",
        digest: "sha256:abcdef1234567890",
        shortDigest: "sha256:abcdef1",
        mediaType: "application/vnd.oci.image.layer.v1.tar+gzip",
        platformOs: "linux",
        platformArch: "amd64",
        sizeBytes: 24_576,
      },
    ],
    about: {
      source: "repository_readme",
      markdown: "# Package README",
      html: "<h1>Package README</h1><p>Install it safely.</p>",
      empty: false,
    },
    admin: {
      canAdmin: true,
      settingsHref: "/ashley/container/opengithub-web/settings",
      reason: null,
    },
  };
  return { ...base, ...overrides };
}

function renderDetail(
  result: PackageDetailFetchResult = {
    ok: true,
    package: packageDetail(),
  },
  ownerKind: "user" | "organization" = "user",
) {
  render(
    <PackageDetailPage owner="ashley" ownerKind={ownerKind} result={result} />,
  );
}

describe("PackageDetailPage", () => {
  it("renders package header, install commands, versions, about, and concrete links", () => {
    renderDetail();

    expect(
      screen.getByRole("heading", { name: "opengithub-web" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Container").length).toBeGreaterThan(0);
    expect(screen.getByText("Latest")).toBeInTheDocument();
    expect(screen.getByText("3,620 downloads")).toBeInTheDocument();
    expect(
      screen.getAllByRole("link", { name: "ashley/opengithub" })[0],
    ).toHaveAttribute("href", "/ashley/opengithub");
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/ashley/container/opengithub-web/settings",
    );
    expect(
      screen.getByText(
        "docker pull ghcr.io/ashley/opengithub-web:1.0.0@sha256:abcdef1234567890",
      ),
    ).toBeInTheDocument();

    const versions = screen.getByRole("heading", { name: "Recent versions" })
      .parentElement?.parentElement;
    expect(versions).not.toBeNull();
    const scoped = within(versions as HTMLElement);
    expect(scoped.getByRole("link", { name: "1.0.0" })).toHaveAttribute(
      "href",
      "/ashley/container/opengithub-web?version=1.0.0",
    );
    expect(scoped.getByRole("link", { name: "0.9.0" })).toHaveAttribute(
      "href",
      "/ashley/container/opengithub-web?version=0.9.0",
    );
    expect(screen.getByRole("heading", { name: "Package README" }));
    expect(screen.getByText("Install it safely.")).toBeInTheDocument();
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders organization canonical links and hides Settings for non-admins", () => {
    renderDetail(
      {
        ok: true,
        package: packageDetail({
          href: "/orgs/namuh/packages/npm/opengithub-web",
          owner: {
            id: "org-1",
            login: "namuh",
            kind: "organization",
            href: "/orgs/namuh",
          },
          linkedRepository: null,
          admin: {
            canAdmin: false,
            settingsHref: null,
            reason: "Package settings require admin access.",
          },
        }),
      },
      "organization",
    );

    expect(screen.getByRole("link", { name: "Packages" })).toHaveAttribute(
      "href",
      "/orgs/namuh/packages",
    );
    expect(screen.getByText("Owner scoped")).toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "Settings" }),
    ).not.toBeInTheDocument();
  });

  it("renders an empty about state without inventing README content", () => {
    renderDetail({
      ok: true,
      package: packageDetail({
        about: {
          source: "none",
          markdown: null,
          html: null,
          empty: true,
        },
      }),
    });

    expect(
      screen.getByText(
        "This package does not have README or about content yet.",
      ),
    ).toBeInTheDocument();
  });

  it("renders forbidden and unavailable states without package metadata leakage", () => {
    renderDetail({
      ok: false,
      status: 403,
      code: "package_not_found",
      message: "package was not found",
    });

    expect(screen.getByText("Read access required")).toBeInTheDocument();
    expect(screen.getByText(/Private and internal package metadata/));
    expect(screen.queryByText("opengithub-web")).not.toBeInTheDocument();

    renderDetail({
      ok: false,
      status: 503,
      code: "packages_unavailable",
      message: "Package detail could not be reached.",
    });

    expect(screen.getByText("Unavailable")).toBeInTheDocument();
    expect(
      screen.getByText("Package detail could not be reached."),
    ).toBeInTheDocument();
  });
});
