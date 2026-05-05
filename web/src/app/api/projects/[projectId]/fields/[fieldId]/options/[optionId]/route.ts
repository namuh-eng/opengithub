import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectFieldOptionFromCookie,
  type ProjectFieldOptionUpdateRequest,
  updateProjectFieldOptionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string; optionId: string }>;
};

const OPTION_COLORS = new Set([
  "gray",
  "red",
  "orange",
  "yellow",
  "green",
  "blue",
  "purple",
  "pink",
]);

function parseOptionRequest(
  input: unknown,
): ProjectFieldOptionUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = typeof body.name === "string" ? body.name.trim() : "";
  const color = typeof body.color === "string" ? body.color.trim() : "gray";
  const description =
    typeof body.description === "string" ? body.description.trim() : null;
  if (!name || !OPTION_COLORS.has(color)) return null;
  return { name, color, description: description || null };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId, optionId } = await context.params;
  const mutation = parseOptionRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Option name and supported color are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateProjectFieldOptionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      decodeURIComponent(optionId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_field_option_update_failed",
          message: "Project field option could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId, optionId } = await context.params;
  try {
    const settings = await deleteProjectFieldOptionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      decodeURIComponent(optionId),
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_field_option_delete_failed",
          message: "Project field option could not be deleted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
