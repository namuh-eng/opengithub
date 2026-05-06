import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AccountSessionsPage } from "@/components/AccountSessionsPage";
import type { AccountSessionsFetchResult } from "@/lib/api";

const sessionsResult: AccountSessionsFetchResult = {
  ok: true,
  sessions: {
    activeCount: 2,
    currentSessionId: "session-current",
    sessions: [
      {
        id: "session-current",
        device: "Mac · Chrome",
        browser: "Chrome",
        location: "Localhost",
        ipAddress: "127.0.0.1",
        userAgent: "Chrome on macOS",
        signedInAt: "2026-05-07T01:00:00Z",
        lastActiveAt: "2026-05-07T01:10:00Z",
        expiresAt: "2026-05-21T01:00:00Z",
        isCurrent: true,
      },
      {
        id: "session-phone",
        device: "iPhone · Safari",
        browser: "Safari",
        location: "Private network",
        ipAddress: "10.1.2.3",
        userAgent: "Safari on iPhone",
        signedInAt: "2026-05-06T01:00:00Z",
        lastActiveAt: "2026-05-06T02:00:00Z",
        expiresAt: "2026-05-20T01:00:00Z",
        isCurrent: false,
      },
    ],
  },
};

describe("AccountSessionsPage", () => {
  it("renders active session metadata and protects the current session", () => {
    const { container } = render(
      <AccountSessionsPage sessionsResult={sessionsResult} />,
    );

    expect(
      screen.getByRole("heading", { name: "Active sessions" }),
    ).toBeVisible();
    expect(screen.getByText("2 active sessions")).toBeVisible();
    expect(screen.getByText("Mac · Chrome")).toBeVisible();
    expect(screen.getByText("iPhone · Safari")).toBeVisible();
    expect(screen.getByText("Current")).toBeVisible();
    const currentRow = screen.getByText("Mac · Chrome").closest("tr");
    expect(currentRow).not.toBeNull();
    expect(
      within(currentRow as HTMLElement).getByRole("button", { name: "Revoke" }),
    ).toBeDisabled();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    expect(container).not.toHaveTextContent("#0969da");
    expect(container).not.toHaveTextContent("Octicon");
  });

  it("revokes an individual non-current session", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        activeCount: 1,
        currentSessionId: "session-current",
        sessions: [sessionsResult.sessions.sessions[0]],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<AccountSessionsPage sessionsResult={sessionsResult} />);

    const phoneRow = screen.getByText("iPhone · Safari").closest("tr");
    expect(phoneRow).not.toBeNull();
    fireEvent.click(
      within(phoneRow as HTMLElement).getByRole("button", { name: "Revoke" }),
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/sessions/actions",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          action: "revoke",
          sessionId: "session-phone",
        }),
      }),
    );
    expect(await screen.findByText("Session revoked.")).toBeVisible();
    expect(screen.queryByText("iPhone · Safari")).not.toBeInTheDocument();
  });

  it("signs out everywhere except the current session", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        activeCount: 1,
        currentSessionId: "session-current",
        sessions: [sessionsResult.sessions.sessions[0]],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<AccountSessionsPage sessionsResult={sessionsResult} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Sign out everywhere" }),
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/sessions/actions",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ action: "sign_out_everywhere" }),
      }),
    );
    expect(
      await screen.findByText("Other sessions have been signed out."),
    ).toBeVisible();
    expect(screen.queryByText("iPhone · Safari")).not.toBeInTheDocument();
  });
});
