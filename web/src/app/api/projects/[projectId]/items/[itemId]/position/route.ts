import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectItemPositionRequest,
  updateProjectItemPositionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

function parsePositionRequest(
  input: unknown,
): ProjectItemPositionRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  return {
    beforeItemId:
      typeof body.beforeItemId === "string" ? body.beforeItemId : null,
    afterItemId: typeof body.afterItemId === "string" ? body.afterItemId : null,
    groupFieldId:
      typeof body.groupFieldId === "string" ? body.groupFieldId : null,
    groupValue: body.groupValue,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  const mutation = parsePositionRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Position update payload is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const workspace = await updateProjectItemPositionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
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
          code: "project_item_position_failed",
          message: "Project item position could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
