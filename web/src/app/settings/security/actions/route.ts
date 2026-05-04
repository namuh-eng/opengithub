import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  createAccountSecuritySudoFromCookie,
  unlinkSignInMethodFromCookie,
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
  const confirmation =
    input && typeof input === "object" && "confirmation" in input
      ? String(input.confirmation)
      : "";
  if (!confirmation.trim()) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Account email is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    return NextResponse.json(
      await createAccountSecuritySudoFromCookie(cookie, { confirmation }),
    );
  } catch (error) {
    return errorResponse(error, {
      code: "sudo_failed",
      message: "Sudo mode could not be enabled.",
      status: 403,
    });
  }
}

export async function DELETE(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  const accountId =
    input && typeof input === "object" && "accountId" in input
      ? String(input.accountId)
      : "";
  if (!accountId) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "accountId is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    return NextResponse.json(
      await unlinkSignInMethodFromCookie(cookie, accountId),
    );
  } catch (error) {
    return errorResponse(error, {
      code: "unlink_failed",
      message: "Sign-in method could not be unlinked.",
      status: 422,
    });
  }
}
