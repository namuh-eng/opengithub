import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectViewStateRequest,
  updateProjectViewStateFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; viewId: string }>;
};

function parseViewStateRequest(input: unknown): ProjectViewStateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const sort = typeof body.sort === "string" ? body.sort.trim() : "";
  const expectedUpdatedAt =
    typeof body.expectedUpdatedAt === "string"
      ? body.expectedUpdatedAt.trim()
      : "";
  if (!sort || !expectedUpdatedAt) return null;
  const hiddenFieldIds = Array.isArray(body.hiddenFieldIds)
    ? body.hiddenFieldIds.filter(
        (value): value is string =>
          typeof value === "string" && value.trim().length > 0,
      )
    : [];
  return {
    query:
      typeof body.query === "string" && body.query.trim()
        ? body.query.trim()
        : null,
    sort,
    group:
      typeof body.group === "string" && body.group.trim()
        ? body.group.trim()
        : null,
    slice:
      typeof body.slice === "string" && body.slice.trim()
        ? body.slice.trim()
        : null,
    hiddenFieldIds,
    expectedUpdatedAt,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, viewId } = await context.params;
  const mutation = parseViewStateRequest(
    await request.json().catch(() => null),
  );
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Sort and expected view timestamp are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const workspace = await updateProjectViewStateFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(viewId),
      mutation,
    );
    return NextResponse.json(workspace);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_view_state_failed",
          message: "Project view state could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
