import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositorySecretScanningAlertFromCookie,
  type RepositorySecretScanningAlertMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

function parseMutation(
  input: unknown,
): RepositorySecretScanningAlertMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  if (body.action === "resolve") {
    const resolution =
      typeof body.resolution === "string" ? body.resolution.trim() : "";
    const resolutionComment =
      typeof body.resolutionComment === "string"
        ? body.resolutionComment.trim()
        : null;
    if (!resolution) return null;
    return { action: "resolve", resolution, resolutionComment };
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
  if (body.action === "validity") {
    const validity =
      typeof body.validity === "string" ? body.validity.trim() : "";
    if (!validity) return null;
    return { action: "validity", validity };
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
          message: "Secret scanning alert action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const detail = await mutateRepositorySecretScanningAlertFromCookie(
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
          code: "secret_scanning_alert_update_failed",
          message: "Secret scanning alert update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
