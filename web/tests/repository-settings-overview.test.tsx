import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositorySettingsOverview } from "@/components/RepositorySettingsOverview";
import type { RepositorySettings } from "@/lib/api";

function settings(
  overrides: Partial<RepositorySettings> = {},
): RepositorySettings {
  return {
    id: "repo-1",
    ownerLogin: "namuh-eng",
    name: "opengithub",
    description: "OpenGitHub",
    visibility: "public",
    defaultBranch: "main",
    isArchived: false,
    isTemplate: false,
    allowForking: true,
    webCommitSignoffRequired: false,
    features: { issues: false, projects: true, wiki: true },
    mergeMethods: {
      mergeCommit: true,
      squash: true,
      rebase: true,
      autoMerge: false,
    },
    capabilities: {
      rename: true,
      archive: false,
      transfer: false,
      changeVisibility: true,
      delete: false,
    },
    viewerPermission: "admin",
    auditEventCount: 0,
    updatedAt: "2026-05-02T00:00:00Z",
    ...overrides,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RepositorySettingsOverview", () => {
  it("persists feature toggles only after the API confirms and reports audit evidence", async () => {
    const next = settings({
      features: { issues: true, projects: true, wiki: true },
      auditEventCount: 1,
      updatedAt: "2026-05-02T01:00:00Z",
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(next), {
        status: 200,
        headers: { "content-type": "application/json" },
      }),
    );

    render(<RepositorySettingsOverview initialSettings={settings()} />);
    const issues = screen.getByRole("checkbox", { name: /Issues/i });
    expect(issues).not.toBeChecked();

    fireEvent.click(issues);

    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/settings",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          features: { issues: true, projects: true, wiki: true },
        }),
      }),
    );
    await waitFor(() => expect(issues).toBeChecked());
    expect(screen.getByText(/Audit event #1 recorded/i)).toBeVisible();
  });

  it("blocks disabling all merge methods before a dead API write", () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    render(
      <RepositorySettingsOverview
        initialSettings={settings({
          mergeMethods: {
            mergeCommit: false,
            squash: false,
            rebase: true,
            autoMerge: false,
          },
        })}
      />,
    );

    fireEvent.click(
      screen.getByRole("checkbox", { name: /Allow rebase merging/i }),
    );

    expect(
      screen.getAllByText(/At least one pull request merge method/i).length,
    ).toBeGreaterThan(0);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("opens typed danger confirmation but leaves final destructive action unavailable", () => {
    render(<RepositorySettingsOverview initialSettings={settings()} />);

    const dangerZone = screen
      .getByRole("heading", { name: "Danger Zone" })
      .closest("section");
    expect(dangerZone).not.toBeNull();
    fireEvent.click(
      within(dangerZone as HTMLElement).getAllByRole("button", {
        name: "Open confirmation",
      })[2],
    );

    expect(
      screen.getByRole("dialog", { name: "Delete repository" }),
    ).toBeVisible();
    fireEvent.change(screen.getByPlaceholderText("namuh-eng/opengithub"), {
      target: { value: "namuh-eng/opengithub" },
    });
    expect(
      screen.getByRole("button", { name: "Delete repository unavailable" }),
    ).toBeDisabled();
  });
});
