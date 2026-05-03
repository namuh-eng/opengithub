import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { revokePersonalAccessTokenFromCookie } from "@/lib/api";

type TokenRouteContext = {
  params: Promise<{ tokenId: string }>;
};

export async function DELETE(_request: Request, context: TokenRouteContext) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const { tokenId } = await context.params;

  try {
    const response = await revokePersonalAccessTokenFromCookie(cookie, tokenId);
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
          code: "token_revoke_failed",
          message: "Personal access token could not be revoked.",
        },
        status: 422,
      },
      { status: envelope?.status ?? 422 },
    );
  }
}
