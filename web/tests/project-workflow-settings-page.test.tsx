import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProjectWorkflowSettingsPage } from "@/components/ProjectWorkflowSettingsPage";
import type { ProjectWorkflowSettings } from "@/lib/api";

function settings(
  overrides: Partial<ProjectWorkflowSettings> = {},
): ProjectWorkflowSettings {
  return {
    project: {
      id: "project-1",
      number: 12,
      title: "Editorial planning",
      description: "Tracks the launch plan.",
      state: "open",
      visibility: "private",
      owner: "namuh",
      href: "/orgs/namuh/projects/12",
      workspaceHref: "/orgs/namuh/projects/12/views/1",
      viewerRole: "write",
    },
    workflows: [
      {
        id: "workflow-closed",
        workflowKey: "closed_item_to_done",
        name: "Set status to Done when closed",
        description:
          "Move linked issues and pull requests into Done when they close.",
        enabled: true,
        triggerEvent: "issue_closed",
        configuration: {
          statusFieldId: "field-status",
          statusOptionId: "option-done",
        },
        rules: [
          {
            id: "rule-1",
            ruleType: "target_status",
            configuration: { optionId: "option-done" },
            position: 1,
          },
        ],
        repositoryTargetIds: ["repo-1"],
        actorLabel: "@opengithub-project-automation",
        source: "system",
        lastRunAt: "2026-05-06T10:15:00Z",
        lastRunStatus: "success",
        lastRunMessage: "Updated 3 items.",
        updatedAt: "2026-05-06T10:15:00Z",
      },
      {
        id: "workflow-archive",
        workflowKey: "auto_archive_done_items",
        name: "Auto-archive completed items",
        description:
          "Archive completed project items after they stay finished long enough.",
        enabled: false,
        triggerEvent: "schedule",
        configuration: { daysAfterCompletion: 14 },
        rules: [],
        repositoryTargetIds: [],
        actorLabel: "@opengithub-project-automation",
        source: "system",
        lastRunAt: null,
        lastRunStatus: null,
        lastRunMessage: null,
        updatedAt: "2026-05-06T09:00:00Z",
      },
    ],
    eligibleFields: [
      {
        id: "field-status",
        name: "Status",
        fieldType: "single_select",
        options: [
          {
            id: "option-done",
            name: "Done",
            color: "green",
            position: 1,
            description: null,
          },
        ],
        supportsStatusTarget: true,
        supportsArchiveCriteria: true,
      },
    ],
    repositoryTargets: [
      {
        id: "repo-1",
        owner: "namuh",
        name: "opengithub",
        fullName: "namuh/opengithub",
        href: "/namuh/opengithub",
        visibility: "private",
        permission: "write",
      },
    ],
    recentLogs: [
      {
        id: "log-1",
        workflowId: "workflow-closed",
        workflowKey: "closed_item_to_done",
        itemId: "item-1",
        actor: null,
        source: "system",
        eventType: "issue_closed",
        status: "success",
        message: "Updated 3 items.",
        metadata: {},
        createdAt: "2026-05-06T10:15:00Z",
      },
    ],
    viewerPermissions: {
      authenticated: true,
      viewerRole: "write",
      canManageWorkflows: true,
      canViewLogs: true,
      canSelectRepositories: true,
    },
    automationActor: "@opengithub-project-automation",
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectWorkflowSettingsPage", () => {
  it("renders the organization workflow settings shell with concrete navigation", () => {
    render(
      <ProjectWorkflowSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Editorial planning" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Back to project" }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "Fields" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/fields",
    );
    expect(screen.getByRole("link", { name: "Workflows" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/workflows",
    );
    expect(
      screen.getByText("1 enabled · @opengithub-project-automation"),
    ).toBeInTheDocument();
  });

  it("renders default cards, activity, and route-backed edit links", () => {
    render(
      <ProjectWorkflowSettingsPage
        owner="mona"
        scope="user"
        settings={settings()}
      />,
    );

    const doneCard = screen
      .getByRole("link", { name: "Set status to Done when closed" })
      .closest("article");
    expect(doneCard).not.toBeNull();
    expect(
      within(doneCard as HTMLElement).getByText("Enabled"),
    ).toBeInTheDocument();
    expect(
      within(doneCard as HTMLElement).getByText("Issue Closed"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Set status to Done when closed" }),
    ).toHaveAttribute(
      "href",
      "/mona/projects/12/workflows?workflow=workflow-closed",
    );
    expect(screen.getAllByText("Updated 3 items.")).toHaveLength(2);

    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    expect(screen.getByText("namuh/opengithub")).toBeInTheDocument();
  });

  it("opens an edit panel without submitting fake mutations", () => {
    render(
      <ProjectWorkflowSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[1]);

    expect(
      screen.getByRole("heading", { name: "Auto-archive completed items" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Event")).toHaveValue("Schedule");
    expect(
      screen.getByRole("button", { name: "Save workflow" }),
    ).toBeDisabled();
    expect(
      screen.getByText("Save is disabled until workflow mutations are added."),
    ).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "#" })).not.toBeInTheDocument();
  });

  it("disables turn-on controls for read-only viewers with explanatory copy", () => {
    render(
      <ProjectWorkflowSettingsPage
        owner="namuh"
        scope="organization"
        selectedWorkflowId="workflow-archive"
        settings={settings({
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canManageWorkflows: false,
            canViewLogs: true,
            canSelectRepositories: false,
          },
        })}
      />,
    );

    expect(screen.getByText("Read-only")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Turn on" })).toBeDisabled();
    expect(
      screen.getByText(
        "You can inspect this workflow, but project write access is required to change it.",
      ),
    ).toBeInTheDocument();
  });

  it("uses Editorial tokens instead of GitHub visual constants", () => {
    const { container } = render(
      <ProjectWorkflowSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
    expect(
      container.querySelectorAll(".chip, .btn, .card").length,
    ).toBeGreaterThan(8);
  });
});
