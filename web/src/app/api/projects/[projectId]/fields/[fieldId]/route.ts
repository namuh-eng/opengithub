import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectFieldFromCookie,
  type ProjectFieldDeleteRequest,
  type ProjectFieldUpdateRequest,
  updateProjectFieldFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
};

function parseUpdateRequest(input: unknown): ProjectFieldUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = typeof body.name === "string" ? body.name.trim() : "";
  if (!name) return null;
  return {
    name,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

function parseDeleteRequest(input: unknown): ProjectFieldDeleteRequest {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return { expectedUpdatedAt: null };
  }
  const body = input as Record<string, unknown>;
  return {
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

function fallbackEnvelope(code: string, message: string) {
  return { error: { code, message }, status: 502 };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
  const mutation = parseUpdateRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Field name is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateProjectFieldFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ??
        fallbackEnvelope(
          "project_field_update_failed",
          "Project field could not be saved.",
        ),
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
  const mutation = parseDeleteRequest(await request.json().catch(() => null));
  try {
    const settings = await deleteProjectFieldFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ??
        fallbackEnvelope(
          "project_field_delete_failed",
          "Project field could not be deleted.",
        ),
      { status: envelope?.status ?? 502 },
    );
  }
}
