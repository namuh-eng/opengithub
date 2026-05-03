import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  createNotificationCustomFilterFromCookie,
  deleteNotificationCustomFilterFromCookie,
  updateNotificationCustomFilterFromCookie,
} from "@/lib/api";

export async function POST(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return invalidJson();
  }

  try {
    const settings = await createNotificationCustomFilterFromCookie(
      cookie,
      input as { name: string; queryString: string },
    );
    return NextResponse.json(settings);
  } catch (error) {
    return filterError(error);
  }
}

export async function PATCH(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return invalidJson();
  }
  const filterId = (input as { id?: unknown }).id;
  if (typeof filterId !== "string" || !filterId.trim()) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Filter id is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateNotificationCustomFilterFromCookie(
      cookie,
      filterId,
      input as { name: string; queryString: string },
    );
    return NextResponse.json(settings);
  } catch (error) {
    return filterError(error);
  }
}

export async function DELETE(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return invalidJson();
  }
  const filterId = (input as { id?: unknown }).id;
  if (typeof filterId !== "string" || !filterId.trim()) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Filter id is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await deleteNotificationCustomFilterFromCookie(
      cookie,
      filterId,
    );
    return NextResponse.json(settings);
  } catch (error) {
    return filterError(error);
  }
}

function invalidJson() {
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

function filterError(error: unknown) {
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
        code: "validation_failed",
        message: "Notification filter could not be saved.",
      },
      status: 422,
    },
    { status: envelope?.status ?? 422 },
  );
}
