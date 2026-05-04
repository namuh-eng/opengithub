import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OrganizationCreatePage } from "@/components/OrganizationCreatePage";

function availability(normalizedSlug: string, available = true) {
  return {
    requestedName: normalizedSlug,
    normalizedSlug,
    available,
    reason: available ? null : "This organization slug is reserved.",
    reserved: !available,
    existingKind: null,
  };
}

function mockFetch(response: unknown, ok = true) {
  return vi.fn().mockResolvedValue({
    json: async () => response,
    ok,
  }) as unknown as typeof fetch;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("OrganizationCreatePage", () => {
  it("renders Editorial plan cards with only Free enabled", () => {
    const { container } = render(<OrganizationCreatePage />);

    expect(
      screen.getByRole("heading", { name: "Create a new organization" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Free plan")).toBeVisible();
    expect(screen.getByLabelText("Team plan")).toBeVisible();
    expect(screen.getByLabelText("Enterprise plan")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Create a free organization" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Team plan unavailable" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Enterprise plan unavailable" }),
    ).toBeDisabled();
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(2);
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("opens setup form and validates slug availability with normalized preview", async () => {
    global.fetch = mockFetch(availability("acme-labs"));

    render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );

    expect(
      screen.getByRole("heading", { name: "Tell us about your organization" }),
    ).toBeVisible();
    fireEvent.change(screen.getByLabelText("Organization name *"), {
      target: { value: "Acme Labs!!" },
    });
    expect(screen.getByText("opengithub.namuh.co/acme-labs")).toBeVisible();

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        "/organizations/new/slug-availability?name=Acme+Labs%21%21",
        expect.objectContaining({ signal: expect.any(AbortSignal) }),
      );
    });
    await waitFor(() => {
      expect(screen.getByText("acme-labs is available.")).toBeVisible();
    });
  });

  it("renders reserved slug, conditional company field, and required terms state", async () => {
    global.fetch = mockFetch(availability("settings", false));

    render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );
    fireEvent.change(screen.getByLabelText("Organization name *"), {
      target: { value: "settings" },
    });
    await waitFor(() => {
      expect(
        screen.getByText("This organization slug is reserved."),
      ).toBeVisible();
    });

    expect(
      screen.getByRole("button", { name: "Create organization" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByLabelText("Business or institution"));
    expect(screen.getByLabelText("Company name *")).toBeVisible();
    fireEvent.click(
      screen.getByLabelText(
        "I accept the organization terms for this Free plan.",
      ),
    );
    expect(
      screen.getByRole("button", { name: "Create organization" }),
    ).toBeDisabled();
  });

  it("has no dead anchors or unnamed buttons", () => {
    const { container } = render(<OrganizationCreatePage />);

    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });
});
