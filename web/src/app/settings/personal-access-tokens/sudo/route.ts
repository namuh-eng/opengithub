import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { createSudoGrantFromCookie } from "@/lib/api";

export async function POST(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  const confirmation =
    "confirmation" in input && typeof input.confirmation === "string"
      ? input.confirmation
      : "";

  try {
    const response = await createSudoGrantFromCookie(cookie, { confirmation });
    return NextResponse.json(response);
  } catch (error) {
    const cause = error instanceof Error ? error.cause : null;
    const envelope =
      cause && typeof cause === "object" && "error" in cause
        ? (cause as {
            error: { code: string; message: string };
            status?: number;
          })
        : null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "sudo_failed",
          message: "Sudo mode could not be enabled.",
        },
        status: 403,
      },
      { status: envelope?.status ?? 403 },
    );
  }
}
