import { describe, expect, it } from "vitest";
import { GET } from "@/app/healthz/route";

describe("web healthz route", () => {
  it("returns a public 200 health response", async () => {
    const response = GET();
    const body = await response.json();

    expect(response.status).toBe(200);
    expect(body).toEqual({ service: "opengithub-web", status: "ok" });
  });
});
