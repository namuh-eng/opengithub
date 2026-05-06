import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AccountSecurityLogPage } from "@/components/AccountSecurityLogPage";
import type { AccountSecurityLogFetchResult } from "@/lib/api";

const logResult: AccountSecurityLogFetchResult = {
  ok: true,
  log: {
    actions: ["session.revoke", "sign_in_method.unlink"],
    events: [
      {
        id: "event-1",
        action: "sign_in_method.unlink",
        location: "Private network",
        ipAddress: "10.1.2.3",
        userAgent: "Safari on iPhone",
        metadata: {},
        createdAt: "2026-05-07T01:20:00Z",
      },
      {
        id: "event-2",
        action: "session.revoke",
        location: "Localhost",
        ipAddress: "127.0.0.1",
        userAgent: "Chrome on macOS",
        metadata: {},
        createdAt: "2026-05-07T01:10:00Z",
      },
    ],
    filters: {
      action: null,
      page: 1,
      pageSize: 50,
    },
    pagination: {
      total: 2,
      page: 1,
      pageSize: 50,
      totalPages: 1,
      hasPrevious: false,
      hasNext: false,
    },
  },
};

describe("AccountSecurityLogPage", () => {
  it("renders security events with filter and attachment export links", () => {
    const { container } = render(
      <AccountSecurityLogPage action={null} logResult={logResult} page={1} />,
    );

    expect(screen.getByRole("heading", { name: "Security log" })).toBeVisible();
    expect(screen.getByText("2 events recorded")).toBeVisible();
    expect(
      screen.getByRole("table", { name: "Security log events" }),
    ).toBeVisible();
    expect(
      screen.getAllByText("sign in method / unlink").length,
    ).toBeGreaterThan(0);
    expect(screen.getAllByText("session / revoke").length).toBeGreaterThan(0);
    expect(screen.getByText("10.1.2.3")).toBeVisible();
    expect(screen.getByText("Private network")).toBeVisible();

    const actionSelect = screen.getByLabelText("Action");
    expect(
      within(actionSelect).getByRole("option", {
        name: "sign in method / unlink",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Export CSV" })).toHaveAttribute(
      "href",
      "/settings/security-log/export?format=csv",
    );
    expect(screen.getByRole("link", { name: "Export JSON" })).toHaveAttribute(
      "href",
      "/settings/security-log/export?format=json",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    expect(container).not.toHaveTextContent("#0969da");
    expect(container).not.toHaveTextContent("Octicon");
  });

  it("preserves the selected filter in pagination and exports", () => {
    render(
      <AccountSecurityLogPage
        action="session.revoke"
        logResult={{
          ok: true,
          log: {
            ...logResult.log,
            filters: { action: "session.revoke", page: 2, pageSize: 50 },
            pagination: {
              total: 60,
              page: 2,
              pageSize: 50,
              totalPages: 2,
              hasPrevious: true,
              hasNext: false,
            },
          },
        }}
        page={2}
      />,
    );

    expect(screen.getByRole("link", { name: "Export CSV" })).toHaveAttribute(
      "href",
      "/settings/security-log/export?format=csv&action=session.revoke",
    );
    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/settings/security-log?action=session.revoke",
    );
  });
});
