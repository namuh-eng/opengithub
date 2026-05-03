import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NotificationDeliverySettingsPage } from "@/components/NotificationDeliverySettingsPage";
import type { NotificationDeliverySettings } from "@/lib/api";

function deliverySettings(
  overrides: Partial<NotificationDeliverySettings> = {},
): NotificationDeliverySettings {
  return {
    defaultEmailId: "email-1",
    defaultEmail: "mona@example.com",
    emailChannelAvailable: true,
    sesSenderReady: true,
    emails: [
      {
        id: "email-1",
        email: "mona@example.com",
        isPrimary: true,
        isPublic: true,
        verified: true,
      },
      {
        id: "email-2",
        email: "draft@example.com",
        isPrimary: false,
        isPublic: false,
        verified: false,
      },
    ],
    preferences: [
      {
        key: "watching",
        label: "Watching",
        section: "subscriptions",
        description: "Repositories you watch directly.",
        channels: ["web"],
        supportedChannels: ["web", "email", "cli"],
        disabled: false,
        disabledReason: null,
      },
      {
        key: "participating",
        label: "Participating, @mentions, and review requests",
        section: "subscriptions",
        description: "Threads where you are participating.",
        channels: ["web", "email"],
        supportedChannels: ["web", "email", "cli"],
        disabled: false,
        disabledReason: null,
      },
      {
        key: "actions",
        label: "Actions",
        section: "system",
        description: "Workflow activity.",
        channels: ["web"],
        supportedChannels: ["web", "email", "cli"],
        disabled: false,
        disabledReason: null,
      },
      {
        key: "dependabot",
        label: "Dependabot",
        section: "system",
        description: "Dependency activity.",
        channels: ["web"],
        supportedChannels: ["web", "email", "cli"],
        disabled: true,
        disabledReason: "Dependabot alerts are not built yet.",
      },
    ],
    customRoutingHref: "/settings/notifications#custom-routing",
    watchedRepositoriesHref: "/notifications/subscriptions?filter=watching",
    ignoredRepositoriesHref: "/notifications/subscriptions?filter=ignored",
    ...overrides,
  };
}

describe("NotificationDeliverySettingsPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders delivery preferences, concrete routing links, and disabled placeholders", () => {
    const { container } = render(
      <NotificationDeliverySettingsPage initialSettings={deliverySettings()} />,
    );

    expect(
      screen.getByRole("heading", { name: "Default notifications email" }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Subscriptions" }),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: "System" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Custom routing" }),
    ).toHaveAttribute("href", "/settings/notifications#custom-routing");
    expect(
      screen.getByRole("link", { name: "Ignored repositories" }),
    ).toHaveAttribute("href", "/notifications/subscriptions?filter=ignored");
    expect(
      screen.getAllByRole("button", { name: "Notify me" })[0],
    ).toBeEnabled();
    const dependabotRow = screen.getByText("Dependabot").closest(".list-row");
    expect(
      within(dependabotRow as HTMLElement).getByRole("button", {
        name: "Notify me",
      }),
    ).toBeDisabled();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("saves the default verified email through the same-origin action", async () => {
    const next = deliverySettings({ defaultEmailId: null, defaultEmail: null });
    const fetchMock = vi.fn(
      async () =>
        new Response(JSON.stringify(next), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
    );
    vi.stubGlobal("fetch", fetchMock);
    render(
      <NotificationDeliverySettingsPage initialSettings={deliverySettings()} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Save email" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/notifications/delivery",
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ defaultEmailId: "email-1" }),
        },
      ),
    );
    expect(
      await screen.findByText("Default notifications email saved."),
    ).toBeVisible();
  });

  it("opens a channel select panel, cancels local edits, then persists selected channels", async () => {
    const saved = deliverySettings({
      preferences: deliverySettings().preferences.map((preference) =>
        preference.key === "watching"
          ? { ...preference, channels: ["web", "email", "cli"] }
          : preference,
      ),
    });
    const fetchMock = vi.fn(
      async () =>
        new Response(JSON.stringify(saved), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
    );
    vi.stubGlobal("fetch", fetchMock);
    render(
      <NotificationDeliverySettingsPage initialSettings={deliverySettings()} />,
    );

    const watchingRow = screen.getByText("Watching").closest(".list-row");
    fireEvent.click(
      within(watchingRow as HTMLElement).getByRole("button", {
        name: "Notify me",
      }),
    );
    let dialog = screen.getByRole("dialog", { name: "Watching" });
    fireEvent.click(within(dialog).getByLabelText("Email"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(fetchMock).not.toHaveBeenCalled();

    fireEvent.click(
      within(watchingRow as HTMLElement).getByRole("button", {
        name: "Notify me",
      }),
    );
    dialog = screen.getByRole("dialog", { name: "Watching" });
    fireEvent.click(within(dialog).getByLabelText("Email"));
    fireEvent.click(within(dialog).getByLabelText("CLI"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/settings/notifications/delivery",
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            defaultEmailId: "email-1",
            preferences: [
              { key: "watching", channels: ["web", "email", "cli"] },
            ],
          }),
        },
      ),
    );
    expect(
      await screen.findByText("Notification channels saved."),
    ).toBeVisible();
    expect(screen.getByText("CLI")).toBeVisible();
  });
});
