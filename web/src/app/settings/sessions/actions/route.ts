import { type NextRequest, NextResponse } from "next/server";
import {
  revokeAccountSessionFromCookie,
  signOutEverywhereFromCookie,
} from "@/lib/api";

export async function POST(request: NextRequest) {
  const cookie = request.headers.get("cookie");
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    body = {};
  }
  const action =
    body && typeof body === "object" && "action" in body
      ? String(body.action)
      : "";

  try {
    if (action === "revoke") {
      const sessionId =
        body && typeof body === "object" && "sessionId" in body
          ? String(body.sessionId)
          : "";
      if (!sessionId) {
        return NextResponse.json(
          {
            error: {
              code: "invalid_session",
              message: "A session id is required.",
            },
            status: 422,
          },
          { status: 422 },
        );
      }
      const response = await revokeAccountSessionFromCookie(cookie, sessionId);
      return NextResponse.json(response.sessions);
    }

    if (action === "sign_out_everywhere") {
      const response = await signOutEverywhereFromCookie(cookie);
      return NextResponse.json(response.sessions);
    }

    return NextResponse.json(
      {
        error: {
          code: "invalid_action",
          message: "The session action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  } catch (error) {
    return NextResponse.json(
      {
        error: {
          code: "session_action_failed",
          message:
            error instanceof Error
              ? error.message
              : "Session action could not be completed.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }
}
