import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  type CreatePersonalAccessTokenRequest,
  createPersonalAccessTokenFromCookie,
} from "@/lib/api";

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

  try {
    const response = await createPersonalAccessTokenFromCookie(
      cookie,
      input as CreatePersonalAccessTokenRequest,
    );
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
          code: "token_create_failed",
          message: "Personal access token could not be created.",
        },
        status: 422,
      },
      { status: envelope?.status ?? 422 },
    );
  }
}
