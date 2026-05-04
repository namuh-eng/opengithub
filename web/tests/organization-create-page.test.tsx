import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { NextRequest } from "next/server";
import { afterEach, describe, expect, it, vi } from "vitest";
import { POST as createOrganizationRoute } from "@/app/organizations/new/create/route";
import { OrganizationCreatePage } from "@/components/OrganizationCreatePage";
import {
  type CreatedOrganization,
  createOrganizationFromCookie,
} from "@/lib/api";

const routerPush = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: routerPush,
  }),
}));

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

function createdOrganization(
  overrides: Partial<CreatedOrganization> = {},
): CreatedOrganization {
  return {
    id: "org-1",
    slug: "acme-labs",
    displayName: "Acme Labs",
    contactEmail: "admin@example.com",
    ownershipType: "business",
    companyName: "Acme Inc.",
    termsOfServiceType: "free_organization_terms",
    role: "owner",
    href: "/orgs/acme-labs",
    settingsHref: "/organizations/acme-labs/settings/profile",
    createdAt: "2026-05-04T00:00:00Z",
    ...overrides,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
  routerPush.mockReset();
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

  it("focuses the first invalid field and exposes accessible descriptions", async () => {
    global.fetch = mockFetch(availability("acme-labs"));

    const { container } = render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );

    const heading = screen.getByRole("heading", {
      name: "Tell us about your organization",
    });
    await waitFor(() => expect(heading).toHaveFocus());

    fireEvent.submit(container.querySelector("form") as HTMLFormElement);
    const nameInput = screen.getByLabelText("Organization name *");
    await waitFor(() => expect(nameInput).toHaveFocus());
    expect(nameInput).toHaveAccessibleDescription(/lowercase hyphenated slug/i);
    expect(nameInput).toHaveAttribute("aria-invalid", "true");

    expect(
      screen.getByRole("group", { name: "Ownership type *" }),
    ).toHaveAccessibleDescription(/business organizations require/i);
    expect(container.querySelector(".min-w-0")).toBeInTheDocument();
    expect(container.querySelector(".break-words")).toBeInTheDocument();
  });

  it("renders API rate-limit envelopes without assigning them to a field", async () => {
    global.fetch = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () => availability("rate-limited-org"),
        ok: true,
      })
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: "rate_limited",
              message: "too many organization creation attempts",
            },
            status: 429,
          }),
          { status: 429 },
        ),
      ) as unknown as typeof fetch;

    render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );
    fireEvent.change(screen.getByLabelText("Organization name *"), {
      target: { value: "Rate Limited Org" },
    });
    await waitFor(() => {
      expect(screen.getByText("rate-limited-org is available.")).toBeVisible();
    });
    fireEvent.change(screen.getByLabelText("Contact email *"), {
      target: { value: "admin@example.com" },
    });
    fireEvent.click(
      screen.getByLabelText(
        "I accept the organization terms for this Free plan.",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Create organization" }),
    );

    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Too many organization creation attempts. Wait a moment, then try again.",
      );
    });
    expect(screen.getByLabelText("Organization name *")).toHaveAttribute(
      "aria-invalid",
      "false",
    );
  });

  it("submits organization creation, disables duplicate submits, and redirects", async () => {
    let resolveCreate: (value: Response) => void = () => {};
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () => availability("acme-labs"),
        ok: true,
      })
      .mockReturnValueOnce(
        new Promise<Response>((resolve) => {
          resolveCreate = resolve;
        }),
      );
    global.fetch = fetchMock as unknown as typeof fetch;

    render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );
    fireEvent.change(screen.getByLabelText("Organization name *"), {
      target: { value: "Acme Labs" },
    });
    await waitFor(() => {
      expect(screen.getByText("acme-labs is available.")).toBeVisible();
    });
    fireEvent.change(screen.getByLabelText("Contact email *"), {
      target: { value: "admin@example.com" },
    });
    fireEvent.click(screen.getByLabelText("Business or institution"));
    fireEvent.change(screen.getByLabelText("Company name *"), {
      target: { value: "Acme Inc." },
    });
    fireEvent.click(
      screen.getByLabelText(
        "I accept the organization terms for this Free plan.",
      ),
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Create organization" }),
    );
    expect(screen.getByRole("button", { name: "Creating..." })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Creating..." }));
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock).toHaveBeenLastCalledWith("/organizations/new/create", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        name: "Acme Labs",
        contactEmail: "admin@example.com",
        ownershipType: "business",
        companyName: "Acme Inc.",
        termsAccepted: true,
      }),
    });

    resolveCreate(
      new Response(JSON.stringify(createdOrganization()), { status: 201 }),
    );
    await waitFor(() =>
      expect(routerPush).toHaveBeenCalledWith("/orgs/acme-labs"),
    );
  });

  it("keeps entered values and renders inline API validation failures", async () => {
    global.fetch = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () => availability("acme-labs"),
        ok: true,
      })
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: "conflict",
              message: "organization slug is already taken",
            },
            status: 409,
          }),
          { status: 409 },
        ),
      ) as unknown as typeof fetch;

    render(<OrganizationCreatePage />);
    fireEvent.click(
      screen.getByRole("button", { name: "Create a free organization" }),
    );
    fireEvent.change(screen.getByLabelText("Organization name *"), {
      target: { value: "Acme Labs" },
    });
    await waitFor(() => {
      expect(screen.getByText("acme-labs is available.")).toBeVisible();
    });
    fireEvent.change(screen.getByLabelText("Contact email *"), {
      target: { value: "admin@example.com" },
    });
    fireEvent.click(
      screen.getByLabelText(
        "I accept the organization terms for this Free plan.",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Create organization" }),
    );

    await waitFor(() => {
      expect(
        screen.getByText("organization slug is already taken"),
      ).toBeVisible();
    });
    expect(screen.getByLabelText("Organization name *")).toHaveValue(
      "Acme Labs",
    );
    expect(screen.getByLabelText("Contact email *")).toHaveValue(
      "admin@example.com",
    );
    expect(routerPush).not.toHaveBeenCalled();
  });
});

describe("organization create API helpers", () => {
  it("creates organizations with the signed session cookie", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(createdOrganization()), {
        status: 201,
      }),
    );
    vi.stubGlobal("fetch", fetchMock);
    vi.stubEnv("API_URL", "http://api.local");

    await expect(
      createOrganizationFromCookie("og_session=value", {
        name: "Acme Labs",
        contactEmail: "admin@example.com",
        ownershipType: "business",
        companyName: "Acme Inc.",
        termsAccepted: true,
      }),
    ).resolves.toMatchObject({ href: "/orgs/acme-labs" });
    expect(fetchMock).toHaveBeenCalledWith(
      "http://api.local/api/organizations",
      {
        method: "POST",
        headers: {
          "content-type": "application/json",
          cookie: "og_session=value",
        },
        body: JSON.stringify({
          name: "Acme Labs",
          contactEmail: "admin@example.com",
          ownershipType: "business",
          companyName: "Acme Inc.",
          termsAccepted: true,
        }),
        cache: "no-store",
      },
    );
  });

  it("proxies organization creation and preserves API error envelopes", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          error: { code: "conflict", message: "organization slug is taken" },
          status: 409,
        }),
        { status: 409 },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);
    vi.stubEnv("API_URL", "http://api.local");

    const request = new Request(
      "http://localhost:3015/organizations/new/create",
      {
        method: "POST",
        headers: {
          cookie: "og_session=value",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          name: "Acme Labs",
          contactEmail: "admin@example.com",
          ownershipType: "personal",
          termsAccepted: true,
        }),
      },
    ) as NextRequest;

    const response = await createOrganizationRoute(request);
    expect(response.status).toBe(409);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "conflict" },
      status: 409,
    });
  });
});
