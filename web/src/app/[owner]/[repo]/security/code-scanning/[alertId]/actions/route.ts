import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryCodeScanningAlertFromCookie,
  type RepositoryCodeScanningAlertMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

function parseMutation(
  input: unknown,
): RepositoryCodeScanningAlertMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  if (body.action === "dismiss") {
    const dismissalReason =
      typeof body.dismissalReason === "string"
        ? body.dismissalReason.trim()
        : "";
    const dismissalComment =
      typeof body.dismissalComment === "string"
        ? body.dismissalComment.trim()
        : null;
    if (!dismissalReason) return null;
    return {
      action: "dismiss",
      dismissalComment,
      dismissalReason,
    };
  }
  if (body.action === "reopen") {
    return { action: "reopen" };
  }
  if (body.action === "assign") {
    const assigneeIds = Array.isArray(body.assigneeIds)
      ? body.assigneeIds.filter(
          (value): value is string => typeof value === "string",
        )
      : [];
    return { action: "assign", assigneeIds };
  }
  if (body.action === "link_issue") {
    const linkedIssueId =
      typeof body.linkedIssueId === "string" ? body.linkedIssueId.trim() : "";
    if (!linkedIssueId) return null;
    return { action: "link_issue", linkedIssueId };
  }
  return null;
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, alertId } = await context.params;
  const input = await request.json().catch(() => null);
  const mutation = parseMutation(input);
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Code scanning alert action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const detail = await mutateRepositoryCodeScanningAlertFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(alertId),
      mutation,
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "code_scanning_alert_update_failed",
          message: "Code scanning alert update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
