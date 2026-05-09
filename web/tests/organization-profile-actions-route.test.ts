import { headers } from "next/headers";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  PATCH,
  POST,
} from "@/app/organizations/[org]/settings/profile/actions/route";
import {
  renameOrganizationFromCookie,
  updateOrganizationProfileSettingsFromCookie,
} from "@/lib/api";

vi.mock("next/headers", () => ({
  headers: vi.fn(),
}));

vi.mock("@/lib/api", () => ({
  renameOrganizationFromCookie: vi.fn(),
  updateOrganizationProfileSettingsFromCookie: vi.fn(),
}));

const mockedHeaders = vi.mocked(headers);
const mockedUpdate = vi.mocked(updateOrganizationProfileSettingsFromCookie);
const mockedRename = vi.mocked(renameOrganizationFromCookie);

function request(body: unknown) {
  return new Request(
    "http://localhost/organizations/acme/settings/profile/actions",
    {
      body: JSON.stringify(body),
      headers: { "content-type": "application/json" },
      method: "PATCH",
    },
  );
}

describe("organization profile actions route", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mockedHeaders.mockResolvedValue(
      new Headers({ cookie: "__Host-session=signed" }) as never,
    );
  });

  it("preserves upstream auth/status errors for profile updates", async () => {
    mockedUpdate.mockRejectedValue(
      new Error("organization settings require owner access", {
        cause: {
          error: { code: "forbidden" },
          status: 403,
        },
      }),
    );

    const response = await PATCH(request({ displayName: "Acme" }), {
      params: Promise.resolve({ org: "acme" }),
    });
    const body = await response.json();

    expect(response.status).toBe(403);
    expect(body).toEqual({
      error: {
        code: "forbidden",
        message: "organization settings require owner access",
      },
      status: 403,
    });
  });

  it("preserves upstream conflict errors for renames", async () => {
    mockedRename.mockRejectedValue(
      new Error("organization slug is already taken", {
        cause: {
          error: { code: "conflict" },
          status: 409,
        },
      }),
    );

    const response = await POST(request({ name: "taken" }), {
      params: Promise.resolve({ org: "acme" }),
    });
    const body = await response.json();

    expect(response.status).toBe(409);
    expect(body.error.code).toBe("conflict");
    expect(body.error.message).toBe("organization slug is already taken");
  });
});
