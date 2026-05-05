import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectFieldOptionFromCookie,
  type ProjectFieldOptionCreateRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
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
): ProjectFieldOptionCreateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = typeof body.name === "string" ? body.name.trim() : "";
  const color = typeof body.color === "string" ? body.color.trim() : "gray";
  const description =
    typeof body.description === "string" ? body.description.trim() : null;
  if (!name || !OPTION_COLORS.has(color)) return null;
  return { name, color, description: description || null };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
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
    const settings = await createProjectFieldOptionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
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
          code: "project_field_option_create_failed",
          message: "Project field option could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
