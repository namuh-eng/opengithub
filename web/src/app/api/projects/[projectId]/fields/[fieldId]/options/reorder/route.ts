import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectFieldOptionReorderRequest,
  reorderProjectFieldOptionsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
};

function parseReorderRequest(
  input: unknown,
): ProjectFieldOptionReorderRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  if (!Array.isArray(body.optionIds)) return null;
  const optionIds = body.optionIds.filter(
    (optionId): optionId is string =>
      typeof optionId === "string" && Boolean(optionId.trim()),
  );
  if (optionIds.length !== body.optionIds.length || optionIds.length === 0) {
    return null;
  }
  return { optionIds };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
  const mutation = parseReorderRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Option reorder must include option ids.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await reorderProjectFieldOptionsFromCookie(
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
      envelope ?? {
        error: {
          code: "project_field_option_reorder_failed",
          message: "Project field options could not be reordered.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
