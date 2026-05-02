import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryWebhookSettingsPage } from "@/components/RepositoryWebhookSettingsPage";
import type {
  RepositoryOverview,
  RepositoryWebhookDetail,
  RepositoryWebhookSettings,
  WebhookDeliveryDetail,
  WebhookDeliverySummary,
} from "@/lib/api";

const pushMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock }),
}));

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
  afterEach(() => {
    vi.restoreAllMocks();
    pushMock.mockReset();
  });

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
    expect(screen.getByRole("button", { name: "Edit" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Test" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Delete" })).toBeEnabled();
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
    expect(screen.getByRole("button", { name: "Redeliver" })).toBeEnabled();
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
        intent="new"
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ hooks: [] }) }}
      />,
    );

    const addEndpoint = screen.getByRole("heading", { name: "Add webhook" });
    expect(addEndpoint).toBeVisible();
    expect(screen.getByLabelText("Payload URL")).toBeVisible();
    expect(screen.getByText("Let me select individual events")).toBeVisible();
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
      expect(button).toHaveAccessibleName(/.+/);
    }
  });

  it("submits create form and updates from confirmed server state", async () => {
    const nextSettings = settings({
      hooks: [
        {
          ...settings().hooks[0],
          id: "hook-created",
          payloadUrl: "https://receiver.example.com/new",
        },
      ],
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue({
      json: async () => ({
        delivery: delivery({ id: "delivery-created" }),
        settings: nextSettings,
      }),
      ok: true,
    } as Response);

    render(
      <RepositoryWebhookSettingsPage
        intent="new"
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ hooks: [] }) }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Payload URL"), {
      target: { value: "https://receiver.example.com/new" },
    });
    fireEvent.click(screen.getByLabelText("Let me select individual events"));
    fireEvent.click(screen.getByLabelText(/Pushes/));
    fireEvent.change(screen.getByLabelText("Secret"), {
      target: { value: "playwright-secret" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add webhook" }));

    await waitFor(() =>
      expect(
        screen.getByText("Webhook created and ping delivery queued."),
      ).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/settings/hooks/actions",
      expect.objectContaining({
        body: expect.stringContaining("https://receiver.example.com/new"),
        method: "POST",
      }),
    );
    expect(
      screen.getByRole("link", {
        name: "https://receiver.example.com/new",
      }),
    ).toBeVisible();
    expect(pushMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/settings/hooks/hook-created?delivery=delivery-created",
    );
  });

  it("requires selected events before submitting local state", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    render(
      <RepositoryWebhookSettingsPage
        intent="new"
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ hooks: [] }) }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Payload URL"), {
      target: { value: "https://receiver.example.com/new" },
    });
    fireEvent.click(screen.getByLabelText("Let me select individual events"));
    fireEvent.click(screen.getByRole("button", { name: "Add webhook" }));

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Select at least one individual event.",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("confirms delete with the hook URL before sending the action", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue({
      json: async () => settings({ hooks: [] }),
      ok: true,
    } as Response);

    render(
      <RepositoryWebhookSettingsPage
        detailResult={{ ok: true, detail: detail() }}
        intent="delete"
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    const deleteButton = screen.getByRole("button", { name: "Delete webhook" });
    expect(deleteButton).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Type payload URL to confirm"), {
      target: { value: "https://receiver.opengithub.local/hook" },
    });
    fireEvent.click(deleteButton);

    await waitFor(() =>
      expect(screen.getByText("Webhook deleted.")).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/settings/hooks/actions",
      expect.objectContaining({
        body: expect.stringContaining("delete-webhook"),
        method: "POST",
      }),
    );
  });
});
