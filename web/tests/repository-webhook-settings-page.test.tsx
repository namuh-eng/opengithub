import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryWebhookSettingsPage } from "@/components/RepositoryWebhookSettingsPage";
import type {
  RepositoryOverview,
  RepositoryWebhookDetail,
  RepositoryWebhookSettings,
  WebhookDeliveryDetail,
  WebhookDeliverySummary,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: null,
    owner_organization_id: "org-1",
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "private",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "admin",
    branchCount: 3,
    tagCount: 1,
    defaultBranchRef: null,
    latestCommit: null,
    rootEntries: [],
    files: [],
    readme: null,
    sidebar: {
      about: null,
      websiteUrl: null,
      topics: [],
      starsCount: 0,
      watchersCount: 0,
      forksCount: 0,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 2,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:namuh-eng/opengithub.git",
      https: "https://opengithub.namuh.co/namuh-eng/opengithub.git",
      zip: "/namuh-eng/opengithub/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function delivery(
  overrides: Partial<WebhookDeliverySummary> = {},
): WebhookDeliverySummary {
  return {
    attemptCount: 1,
    createdAt: "2026-05-03T02:00:00Z",
    deliveredAt: "2026-05-03T02:00:01Z",
    durationMs: 88,
    event: "push",
    guid: "00000000-0000-4000-8000-000000000001",
    id: "delivery-1",
    redeliveryOfId: null,
    responseStatus: 200,
    status: "delivered",
    updatedAt: "2026-05-03T02:00:01Z",
    ...overrides,
  };
}

function settings(
  overrides: Partial<RepositoryWebhookSettings> = {},
): RepositoryWebhookSettings {
  return {
    canEdit: true,
    eventDefinitions: [
      {
        description: "Git branch and tag updates.",
        label: "Pushes",
        name: "push",
      },
      {
        description: "Issue open, edit, close, label, and comment activity.",
        label: "Issues",
        name: "issues",
      },
      {
        description: "Pull request lifecycle and review activity.",
        label: "Pull requests",
        name: "pull_request",
      },
    ],
    hooks: [
      {
        active: true,
        contentType: "json",
        createdAt: "2026-05-03T00:00:00Z",
        disabledReason: null,
        eventSelection: "selected",
        events: ["push", "issues"],
        id: "hook-1",
        latestDelivery: delivery(),
        payloadUrl: "https://receiver.opengithub.local/hook",
        secretConfigured: true,
        secretUpdatedAt: "2026-05-03T00:00:00Z",
        sslVerify: true,
        updatedAt: "2026-05-03T01:00:00Z",
      },
    ],
    name: "opengithub",
    ownerLogin: "namuh-eng",
    repositoryId: "repo-1",
    viewerPermission: "admin",
    visibility: "private",
    ...overrides,
  };
}

function detail(
  overrides: Partial<RepositoryWebhookDetail> = {},
): RepositoryWebhookDetail {
  return {
    deliveries: [
      delivery(),
      delivery({
        attemptCount: 4,
        durationMs: 321,
        event: "issues",
        guid: "00000000-0000-4000-8000-000000000002",
        id: "delivery-2",
        responseStatus: 500,
        status: "failed",
      }),
    ],
    hook: settings().hooks[0],
    ...overrides,
  };
}

function deliveryDetail(
  overrides: Partial<WebhookDeliveryDetail> = {},
): WebhookDeliveryDetail {
  return {
    requestBodyExcerpt: '{"zen":"Keep it logically awesome."}',
    requestBodyStorageKey: null,
    requestHeaders: {
      "x-opengithub-delivery": "00000000-0000-4000-8000-000000000001",
      "x-opengithub-event": "push",
    },
    responseBodyExcerpt: '{"ok":true}',
    responseBodyStorageKey: null,
    responseHeaders: {
      "content-type": "application/json",
    },
    summary: delivery(),
    terminalError: null,
    ...overrides,
  };
}

describe("repository webhook settings page", () => {
  it("renders the empty state with a concrete add webhook route", () => {
    const { container } = render(
      <RepositoryWebhookSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ hooks: [] }) }}
      />,
    );

    expect(screen.getByText("No webhooks")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Add webhook" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks?new=webhook",
    );
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs",
    );
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(0);
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("renders hook rows with status, event summaries, and concrete controls", () => {
    render(
      <RepositoryWebhookSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    expect(screen.getByText("1 webhooks")).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "https://receiver.opengithub.local/hook",
      }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/hooks/hook-1");
    expect(screen.getByText("Active")).toBeVisible();
    expect(screen.getByText("Secret configured")).toBeVisible();
    expect(screen.getByText("Pushes, Issues")).toBeVisible();
    expect(screen.getByText("delivered · 200 · 88ms")).toBeVisible();
    expect(screen.getByRole("link", { name: "Edit" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks/hook-1?edit=webhook",
    );
    expect(screen.getByRole("link", { name: "Test" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks/hook-1?test=ping",
    );
    expect(screen.getByRole("link", { name: "Delete" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks/hook-1?delete=confirm",
    );
  });

  it("renders hook detail deliveries and selected delivery panels", () => {
    render(
      <RepositoryWebhookSettingsPage
        deliveryResult={{ ok: true, delivery: deliveryDetail() }}
        detailResult={{ ok: true, detail: detail() }}
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    expect(screen.getByRole("heading", { name: /receiver/ })).toBeVisible();
    expect(screen.getByText("Recent deliveries")).toBeVisible();
    expect(screen.getByText("Configuration")).toBeVisible();
    expect(
      screen.getByText("00000000-0000-4000-8000-000000000002"),
    ).toBeVisible();
    expect(screen.getByText("Response")).toBeVisible();
    expect(screen.getByText(/Keep it logically awesome/)).toBeVisible();
    expect(screen.getByRole("link", { name: "Redeliver" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks/hook-1?delivery=delivery-1&redeliver=confirm",
    );
  });

  it("renders forbidden without leaking private hook URLs", () => {
    render(
      <RepositoryWebhookSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          code: "forbidden",
          message: "user does not have repository admin access",
          ok: false,
          status: 403,
        }}
      />,
    );

    expect(screen.getByText("Webhook settings are restricted")).toBeVisible();
    expect(screen.getByText("Admin access required")).toBeVisible();
    expect(screen.queryByText(/receiver.opengithub.local/)).toBeNull();
  });

  it("renders the new webhook event contract view", () => {
    render(
      <RepositoryWebhookSettingsPage
        mode="new"
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ hooks: [] }) }}
      />,
    );

    const addEndpoint = screen.getByRole("heading", { name: "Add endpoint" });
    expect(addEndpoint).toBeVisible();
    expect(screen.getByText("Pull requests")).toBeVisible();
    expect(screen.getByText("pull_request")).toBeVisible();
    expect(screen.getByRole("link", { name: "Back to hooks" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/hooks",
    );
  });

  it("does not render inert anchors or enabled placeholder buttons", () => {
    const { container } = render(
      <RepositoryWebhookSettingsPage
        deliveryResult={{ ok: true, delivery: deliveryDetail() }}
        detailResult={{ ok: true, detail: detail() }}
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    for (const anchor of Array.from(container.querySelectorAll("a"))) {
      expect(anchor.getAttribute("href")).toBeTruthy();
      expect(anchor.getAttribute("href")).not.toBe("#");
    }

    for (const button of Array.from(container.querySelectorAll("button"))) {
      expect(button).toBeDisabled();
    }
  });
});
