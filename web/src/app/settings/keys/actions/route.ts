import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  type CreateSshKeyRequest,
  createSshKeyFromCookie,
  revokeSshKeyFromCookie,
} from "@/lib/api";

function errorResponse(
  error: unknown,
  fallback: { code: string; message: string; status: number },
) {
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
        code: fallback.code,
        message: fallback.message,
      },
      status: fallback.status,
    },
    { status: envelope?.status ?? fallback.status },
  );
}

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
    const response = await createSshKeyFromCookie(
      cookie,
      input as CreateSshKeyRequest,
    );
    return NextResponse.json(response);
  } catch (error) {
    return errorResponse(error, {
      code: "ssh_key_create_failed",
      message: "SSH key could not be added.",
      status: 422,
    });
  }
}

export async function DELETE(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  const keyId =
    input && typeof input === "object" && "keyId" in input
      ? String(input.keyId)
      : "";
  if (!keyId) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "keyId is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const response = await revokeSshKeyFromCookie(cookie, keyId);
    return NextResponse.json(response);
  } catch (error) {
    return errorResponse(error, {
      code: "ssh_key_delete_failed",
      message: "SSH key could not be deleted.",
      status: 422,
    });
  }
}
