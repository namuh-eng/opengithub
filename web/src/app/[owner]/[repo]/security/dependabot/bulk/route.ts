import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  bulkMutateRepositoryDependabotAlertsFromCookie,
  type RepositoryDependabotBulkMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function parseMutation(
  input: unknown,
): RepositoryDependabotBulkMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action =
    body.action === "dismiss" || body.action === "reopen" ? body.action : null;
  const alertIds = Array.isArray(body.alertIds)
    ? body.alertIds.filter(
        (value): value is string => typeof value === "string",
      )
    : [];
  if (!action || alertIds.length === 0) {
    return null;
  }
  const dismissalReason =
    typeof body.dismissalReason === "string"
      ? body.dismissalReason.trim()
      : null;
  const dismissalComment =
    typeof body.dismissalComment === "string"
      ? body.dismissalComment.trim()
      : null;
  if (action === "dismiss" && !dismissalReason) {
    return null;
  }
  return {
    action,
    alertIds,
    dismissalComment,
    dismissalReason,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const input = await request.json().catch(() => null);
  const mutation = parseMutation(input);
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Dependabot bulk action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const result = await bulkMutateRepositoryDependabotAlertsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      mutation,
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "dependabot_bulk_update_failed",
          message: "Dependabot bulk update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
