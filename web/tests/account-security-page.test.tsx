import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AccountSecurityPage } from "@/components/AccountSecurityPage";
import type { AccountSecuritySettingsFetchResult } from "@/lib/api";

const oneMethod: AccountSecuritySettingsFetchResult = {
  ok: true,
  settings: {
    signInMethods: [
      {
        id: "google-1",
        provider: "google",
        displayLabel: "Google",
        email: "owner@opengithub.local",
        avatarUrl: null,
        linkedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
        canUnlink: false,
      },
    ],
    sudo: {
      active: false,
      expiresAt: null,
      requiredFor: ["link_google_account", "unlink_sign_in_method"],
    },
    twoFactor: {
      enabled: false,
      available: false,
      reason:
        "Two-factor authentication is planned after Google-only auth hardening.",
    },
  },
};

const twoMethods: AccountSecuritySettingsFetchResult = {
  ok: true,
  settings: {
    ...oneMethod.settings,
    signInMethods: [
      { ...oneMethod.settings.signInMethods[0], canUnlink: true },
      {
        id: "google-2",
        provider: "google",
        displayLabel: "Google",
        email: "second@opengithub.local",
        avatarUrl: null,
        linkedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
        canUnlink: true,
      },
    ],
    sudo: {
      active: true,
      expiresAt: "2026-05-04T12:30:00Z",
      requiredFor: ["link_google_account", "unlink_sign_in_method"],
    },
  },
};

describe("AccountSecurityPage", () => {
  it("renders linked Google account state and disables last identity unlink", () => {
    const { container } = render(
      <AccountSecurityPage
        linkGoogleHref="http://localhost:3016/api/settings/security/google/link?next=/settings/security"
        securitySettings={oneMethod}
        userEmail="owner@opengithub.local"
      />,
    );

    expect(screen.getByRole("heading", { name: "Security" })).toBeVisible();
    expect(screen.getByText("Sign-in methods")).toBeVisible();
    expect(screen.getByText("owner@opengithub.local")).toBeVisible();
    expect(screen.getByText("Last identity")).toBeVisible();
    expect(screen.getByRole("button", { name: "Unlink" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Configure 2FA" }),
    ).toBeDisabled();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    expect(container).not.toHaveTextContent("#0969da");
    expect(container).not.toHaveTextContent("Octicon");
  });

  it("enables sudo by confirming the account email", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        ...oneMethod.settings,
        sudo: {
          active: true,
          expiresAt: "2026-05-04T12:30:00Z",
          requiredFor: ["link_google_account", "unlink_sign_in_method"],
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <AccountSecurityPage
        linkGoogleHref="http://localhost:3016/api/settings/security/google/link?next=/settings/security"
        securitySettings={oneMethod}
        userEmail="owner@opengithub.local"
      />,
    );

    fireEvent.change(screen.getByLabelText("Account email"), {
      target: { value: "owner@opengithub.local" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Enable sudo" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/security/actions",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ confirmation: "owner@opengithub.local" }),
      }),
    );
    expect(
      await screen.findByText("Sudo mode is active for this session."),
    ).toBeVisible();
    expect(screen.getByText("Sudo active")).toBeVisible();
  });

  it("requires exact email confirmation before unlinking a second sign-in method", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        removedId: "google-2",
        settings: {
          ...oneMethod.settings,
          sudo: twoMethods.settings.sudo,
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <AccountSecurityPage
        linkGoogleHref="http://localhost:3016/api/settings/security/google/link?next=/settings/security"
        securitySettings={twoMethods}
        userEmail="owner@opengithub.local"
      />,
    );

    const row = screen
      .getByText("second@opengithub.local")
      .closest(".list-row");
    expect(row).not.toBeNull();
    fireEvent.click(
      within(row as HTMLElement).getByRole("button", { name: "Unlink" }),
    );
    expect(
      screen.getByRole("button", { name: "Unlink sign-in method" }),
    ).toBeDisabled();
    fireEvent.change(
      screen.getByLabelText("Confirm unlink second@opengithub.local"),
      {
        target: { value: "second@opengithub.local" },
      },
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Unlink sign-in method" }),
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/security/actions",
      expect.objectContaining({
        method: "DELETE",
        body: JSON.stringify({ accountId: "google-2" }),
      }),
    );
    expect(await screen.findByText("Google account unlinked.")).toBeVisible();
    expect(
      screen.queryByText("second@opengithub.local"),
    ).not.toBeInTheDocument();
  });
});
