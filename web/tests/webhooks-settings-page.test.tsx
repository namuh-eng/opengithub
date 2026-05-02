import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WebhooksSettingsPage } from "@/components/WebhooksSettingsPage";
import type { WebhookCatalog } from "@/lib/api";

const catalog: WebhookCatalog = {
  supportedEvents: ["push", "pull_request", "issues", "ping"],
  hooks: [
    {
      id: "hook-1",
      scopeType: "repository",
      scopeId: "repo-1",
      repositoryId: "repo-1",
      url: "https://example.com/hook",
      contentType: "json",
      hasSecret: true,
      events: ["push"],
      active: true,
      sslVerify: true,
      createdByUserId: "user-1",
      createdAt: "2026-05-03T00:00:00Z",
      updatedAt: "2026-05-03T00:00:00Z",
      deliveries: [
        {
          id: "delivery-1",
          webhookId: "hook-1",
          event: "push",
          requestHeaders: { "x-hub-signature-256": "sha256=test" },
          requestBody: JSON.stringify({ ref: "refs/heads/main" }),
          responseStatus: 200,
          responseHeaders: {},
          responseBody: "ok",
          durationMs: 42,
          redeliveryOf: null,
          deliveredAt: "2026-05-03T00:00:01Z",
          status: "delivered",
          attemptCount: 1,
          nextAttemptAt: null,
          createdAt: "2026-05-03T00:00:00Z",
          updatedAt: "2026-05-03T00:00:01Z",
        },
      ],
    },
  ],
};

describe("WebhooksSettingsPage", () => {
  it("renders event catalog, hooks table, and delivery payload viewer", () => {
    render(
      <WebhooksSettingsPage
        catalog={catalog}
        endpointBase="/api/repos/acme/widgets/hooks"
        ownerLabel="acme / widgets"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Webhooks for acme / widgets" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Pull request")).toBeInTheDocument();
    expect(screen.getByText("https://example.com/hook")).toBeInTheDocument();
    expect(screen.getAllByText(/x-hub-signature-256/i).length).toBeGreaterThan(
      0,
    );
  });

  it("creates hooks with selected events and toggles active state", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          ...catalog.hooks[0],
          id: "hook-2",
          url: "https://receiver.test/hook",
          deliveries: [],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ ...catalog.hooks[0], active: false }),
      });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <WebhooksSettingsPage
        catalog={catalog}
        endpointBase="/api/repos/acme/widgets/hooks"
        ownerLabel="acme / widgets"
      />,
    );
    fireEvent.change(screen.getByLabelText("Payload URL"), {
      target: { value: "https://receiver.test/hook" },
    });
    fireEvent.click(screen.getByLabelText("Let me select"));
    fireEvent.click(screen.getByLabelText("Issues"));
    fireEvent.click(screen.getByRole("button", { name: "Add webhook" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/acme/widgets/hooks",
        expect.objectContaining({ method: "POST" }),
      ),
    );
    expect(JSON.parse(fetchMock.mock.calls[0][1].body).events).toContain(
      "issues",
    );

    fireEvent.click(screen.getAllByLabelText("Active")[0]);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/acme/widgets/hooks/hook-2",
        expect.objectContaining({ method: "PATCH" }),
      ),
    );
    vi.unstubAllGlobals();
  });
});
