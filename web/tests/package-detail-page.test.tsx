import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PackageDetailPage } from "@/components/PackageDetailPage";
import { PackageSettingsPage } from "@/components/PackageSettingsPage";
import type {
  PackageDetail,
  PackageDetailFetchResult,
  PackageSettings,
  PackageSettingsFetchResult,
} from "@/lib/api";

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

function packageSettings(
  overrides: Partial<PackageSettings> = {},
): PackageSettings {
  const detail = packageDetail();
  const linkedRepository = detail.linkedRepository ?? {
    id: "repo-1",
    owner: "ashley",
    name: "opengithub",
    fullName: "ashley/opengithub",
    href: "/ashley/opengithub",
    visibility: "public",
  };
  return {
    package: {
      id: detail.id,
      name: detail.name,
      packageType: detail.packageType,
      typeLabel: detail.typeLabel,
      visibility: detail.visibility,
      href: detail.href,
      downloadCount: detail.downloadCount,
      latestVersion: detail.selectedVersion?.version ?? null,
      latestDigest: detail.selectedVersion?.digest ?? null,
      updatedAt: detail.updatedAt,
    },
    owner: detail.owner,
    linkedRepositories: [linkedRepository],
    explicitPermissions: [
      {
        userId: "user-2",
        login: "mona",
        displayName: "Mona Admin",
        role: "admin",
        href: "/mona",
        grantedAt: "2026-05-02T00:00:00Z",
      },
    ],
    inheritedRepositoryAccess: [
      {
        repository: linkedRepository,
        userId: "user-3",
        login: "octo",
        role: "write",
        source: "direct",
        href: "/octo",
      },
    ],
    recentActivity: [
      {
        kind: "version",
        label: "Published 1.0.0",
        actor: detail.publisher,
        occurredAt: "2026-05-01T00:00:00Z",
      },
    ],
    registryWriteCapabilities: [
      {
        key: "visibility",
        label: "Change package visibility",
        enabled: false,
        reason: "Visibility writes are reserved for packages-003.",
      },
    ],
    admin: detail.admin,
    ...overrides,
  };
}

function renderSettings(
  result: PackageSettingsFetchResult = {
    ok: true,
    settings: packageSettings(),
  },
  ownerKind: "user" | "organization" = "user",
) {
  render(
    <PackageSettingsPage
      owner="ashley"
      ownerKind={ownerKind}
      result={result}
    />,
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
      "/ashley/container/opengithub-web?version=sha256%3Aabcdef1234567890",
    );
    expect(scoped.getByRole("link", { name: "0.9.0" })).toHaveAttribute(
      "href",
      "/ashley/container/opengithub-web?version=sha256%3A9999999999999999",
    );
    expect(screen.getByRole("heading", { name: "Package README" }));
    expect(screen.getByText("Install it safely.")).toBeInTheDocument();
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("switches selected version metadata and copies install commands", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    renderDetail();

    fireEvent.change(screen.getByLabelText("Package version"), {
      target: { value: "version-2" },
    });

    expect(
      screen.getByText(
        "docker pull ghcr.io/ashley/opengithub-web:0.9.0@sha256:9999999999999999",
      ),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Selected")).toHaveLength(1);
    fireEvent.click(
      screen.getByRole("button", { name: "Copy install command" }),
    );

    expect(writeText).toHaveBeenCalledWith(
      "docker pull ghcr.io/ashley/opengithub-web:0.9.0@sha256:9999999999999999",
    );
    expect(await screen.findByText("Command copied")).toBeInTheDocument();
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

describe("PackageSettingsPage", () => {
  it("renders admin-only settings state, provenance, disabled write reasons, and concrete links", () => {
    renderSettings();

    expect(
      screen.getByRole("heading", { name: "opengithub-web" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "View package" })).toHaveAttribute(
      "href",
      "/ashley/container/opengithub-web",
    );
    expect(screen.getByRole("link", { name: "Mona Admin" })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(
      screen.getAllByRole("link", { name: "ashley/opengithub" })[0],
    ).toHaveAttribute("href", "/ashley/opengithub");
    expect(screen.getByText("Source: direct")).toBeInTheDocument();
    expect(screen.getByText("Published 1.0.0")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Not available" }),
    ).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText(/reserved for packages-003/)).toBeInTheDocument();
    expect(document.body).not.toHaveTextContent("s3://");
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders organization settings links and empty access states", () => {
    renderSettings(
      {
        ok: true,
        settings: packageSettings({
          owner: {
            id: "org-1",
            login: "namuh",
            kind: "organization",
            href: "/orgs/namuh",
          },
          explicitPermissions: [],
          inheritedRepositoryAccess: [],
        }),
      },
      "organization",
    );

    expect(screen.getByRole("link", { name: "Packages" })).toHaveAttribute(
      "href",
      "/orgs/ashley/packages",
    );
    expect(screen.getByText(/No direct package grants/)).toBeInTheDocument();
    expect(
      screen.getByText(/No inherited repository permissions/),
    ).toBeInTheDocument();
  });

  it("redacts package metadata from forbidden settings responses", () => {
    renderSettings({
      ok: false,
      status: 403,
      code: "forbidden",
      message: "package settings require admin access",
    });

    expect(screen.getByText("Admin access required")).toBeInTheDocument();
    expect(
      screen.getByText(/visible only to package admins/),
    ).toBeInTheDocument();
    expect(screen.queryByText("opengithub-web")).not.toBeInTheDocument();
  });
});
