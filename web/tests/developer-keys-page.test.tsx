import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DeveloperKeysPage } from "@/components/DeveloperKeysPage";
import type { KeySettingsFetchResult } from "@/lib/api";

const validSshPublicKey =
  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIPhY2XwcvYPGAilZzICTAgSiG3kOTaMAP1+y/4U9HQb6 phase2@example";

const emptyKeySettings: KeySettingsFetchResult = {
  ok: true,
  settings: {
    gpgKeys: [],
    sshKeys: [],
    sudo: {
      active: false,
      expiresAt: null,
      requiredFor: ["revoke_ssh_key", "revoke_gpg_key"],
    },
    vigilantMode: false,
  },
};

const populatedKeySettings: KeySettingsFetchResult = {
  ok: true,
  settings: {
    gpgKeys: [],
    sshKeys: [
      {
        id: "ssh-key-1",
        title: "Work laptop",
        keyType: "ssh-ed25519",
        fingerprintSha256: "SHA256:wYVDj6oTzHo4hT2yZGsq7BbuNQjuYHgHuhbzTDm7pIY",
        accessMode: "read_write",
        source: "user_upload",
        lastUsedAt: null,
        revokedAt: null,
        createdAt: "2026-05-04T00:00:00Z",
      },
    ],
    sudo: {
      active: true,
      expiresAt: "2026-05-04T12:30:00Z",
      requiredFor: ["revoke_ssh_key", "revoke_gpg_key"],
    },
    vigilantMode: false,
  },
};

describe("DeveloperKeysPage", () => {
  it("renders an Editorial SSH keys empty state with no inert controls", () => {
    const { container } = render(
      <DeveloperKeysPage keySettings={emptyKeySettings} />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "SSH keys" }),
    ).toBeVisible();
    expect(screen.getByText("No SSH keys yet")).toBeVisible();
    expect(screen.getAllByRole("button", { name: "New SSH key" })).toHaveLength(
      2,
    );
    expect(screen.getByText("GPG keys")).toBeVisible();
    expect(screen.getByText("Vigilant mode")).toBeVisible();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    expect(container).not.toHaveTextContent("#0969da");
    expect(container).not.toHaveTextContent("Octicon");
  });

  it("renders server-backed SSH key metadata without private key material", () => {
    const { container } = render(
      <DeveloperKeysPage keySettings={populatedKeySettings} />,
    );

    expect(screen.getByText("Work laptop")).toBeVisible();
    expect(
      screen.getByText("SHA256:wYVDj6oTzHo4hT2yZGsq7BbuNQjuYHgHuhbzTDm7pIY"),
    ).toBeVisible();
    expect(screen.getByText("Ed25519")).toBeVisible();
    expect(screen.getByText("Read/write")).toBeVisible();
    expect(
      screen.getByText("Sudo mode is active for destructive key changes."),
    ).toBeVisible();
    expect(container).not.toHaveTextContent("PRIVATE KEY");
    expect(container).not.toHaveTextContent(validSshPublicKey);
  });

  it("adds an SSH key through the same-origin action route", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        sshKey: {
          ...populatedKeySettings.settings.sshKeys[0],
          id: "ssh-key-2",
          title: "Build runner",
          fingerprintSha256: "SHA256:phase2created",
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<DeveloperKeysPage keySettings={emptyKeySettings} />);

    fireEvent.click(screen.getAllByRole("button", { name: "New SSH key" })[0]);
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Build runner" },
    });
    fireEvent.change(screen.getByLabelText("Public key"), {
      target: { value: validSshPublicKey },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add SSH key" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/keys/actions",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining(validSshPublicKey),
      }),
    );
    expect(await screen.findByText("Build runner added.")).toBeVisible();
    expect(screen.getByText("SHA256:phase2created")).toBeVisible();
  });

  it("requires delete confirmation before revoking an SSH key", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        revokedAt: "2026-05-04T01:00:00Z",
        sshKey: {
          ...populatedKeySettings.settings.sshKeys[0],
          revokedAt: "2026-05-04T01:00:00Z",
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<DeveloperKeysPage keySettings={populatedKeySettings} />);

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(screen.getByText("Delete Work laptop")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Delete SSH key" }),
    ).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Confirm delete Work laptop"), {
      target: { value: "Work laptop" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Delete SSH key" }));

    expect(fetchMock).toHaveBeenCalledWith(
      "/settings/keys/actions",
      expect.objectContaining({
        method: "DELETE",
        body: JSON.stringify({ keyId: "ssh-key-1" }),
      }),
    );
    expect(await screen.findByText("Work laptop deleted.")).toBeVisible();
    expect(screen.getByText("Deleted")).toBeVisible();
  });

  it("renders unavailable and unauthorized states with concrete sign-in link", () => {
    const { container } = render(
      <DeveloperKeysPage
        keySettings={{
          ok: false,
          status: 401,
          code: "not_authenticated",
          message: "No active session is available",
        }}
      />,
    );

    expect(screen.getByText("SSH keys could not be loaded.")).toBeVisible();
    expect(screen.getByText("Sign in to manage SSH keys.")).toBeVisible();
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      "/login?next=/settings/keys",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });
});
