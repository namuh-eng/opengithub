import { headers } from "next/headers";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DELETE, PATCH } from "@/app/settings/keys/actions/route";
import {
  revokeGpgKeyFromCookie,
  revokeSshKeyFromCookie,
  updateVigilantModeFromCookie,
} from "@/lib/api";

vi.mock("next/headers", () => ({
  headers: vi.fn(),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    revokeGpgKeyFromCookie: vi.fn(),
    revokeSshKeyFromCookie: vi.fn(),
    updateVigilantModeFromCookie: vi.fn(),
  };
});

const mockedHeaders = vi.mocked(headers);
const mockedUpdateVigilantMode = vi.mocked(updateVigilantModeFromCookie);
const mockedRevokeGpgKey = vi.mocked(revokeGpgKeyFromCookie);
const mockedRevokeSshKey = vi.mocked(revokeSshKeyFromCookie);

function jsonRequest(method: string, body: unknown) {
  return new Request("http://localhost/settings/keys/actions", {
    method,
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}

describe("settings key action routes", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedHeaders.mockResolvedValue({
      get: (name: string) => (name === "cookie" ? "og_session=test" : null),
    } as Awaited<ReturnType<typeof headers>>);
  });

  it("rejects non-boolean vigilant mode payloads instead of coercing them", async () => {
    const response = await PATCH(jsonRequest("PATCH", { enabled: "false" }));
    const body = await response.json();

    expect(response.status).toBe(422);
    expect(body.error.code).toBe("validation_failed");
    expect(body.error.message).toBe("enabled must be a boolean.");
    expect(mockedUpdateVigilantMode).not.toHaveBeenCalled();
  });

  it("forwards boolean vigilant mode updates with the session cookie", async () => {
    mockedUpdateVigilantMode.mockResolvedValue({ vigilantMode: false });

    const response = await PATCH(jsonRequest("PATCH", { enabled: false }));
    const body = await response.json();

    expect(response.status).toBe(200);
    expect(body.vigilantMode).toBe(false);
    expect(mockedUpdateVigilantMode).toHaveBeenCalledWith("og_session=test", {
      enabled: false,
    });
  });

  it("rejects unknown signing-key delete kinds", async () => {
    const response = await DELETE(
      jsonRequest("DELETE", { keyId: "key-1", keyKind: "totp" }),
    );
    const body = await response.json();

    expect(response.status).toBe(422);
    expect(body.error.code).toBe("validation_failed");
    expect(body.error.message).toBe("keyKind must be ssh or gpg.");
    expect(mockedRevokeSshKey).not.toHaveBeenCalled();
    expect(mockedRevokeGpgKey).not.toHaveBeenCalled();
  });
});
