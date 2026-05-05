import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectFieldFromCookie,
  type ProjectFieldCreateRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

const FIELD_TYPES = new Set([
  "single_select",
  "iteration",
  "date",
  "text",
  "number",
]);

function parseCreateRequest(input: unknown): ProjectFieldCreateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = typeof body.name === "string" ? body.name.trim() : "";
  const fieldType =
    typeof body.fieldType === "string" ? body.fieldType.trim() : "";
  if (!name || !FIELD_TYPES.has(fieldType)) return null;
  return {
    name,
    fieldType: fieldType as ProjectFieldCreateRequest["fieldType"],
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseCreateRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Field name and supported type are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await createProjectFieldFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      mutation,
    );
    return NextResponse.json(settings, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_field_create_failed",
          message: "Project field could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
