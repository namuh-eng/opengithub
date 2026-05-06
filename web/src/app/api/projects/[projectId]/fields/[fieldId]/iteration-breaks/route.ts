import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectIterationBreakFromCookie,
  type ProjectIterationBreakCreateRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
};

function parseRequest(
  input: unknown,
): ProjectIterationBreakCreateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const startDate = typeof body.startDate === "string" ? body.startDate : "";
  if (!startDate) return null;
  return {
    name: typeof body.name === "string" ? body.name : null,
    startDate,
    durationDays:
      typeof body.durationDays === "number" ? body.durationDays : null,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Break start date is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const settings = await createProjectIterationBreakFromCookie(
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
          code: "project_iteration_break_create_failed",
          message: "Project iteration break could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
