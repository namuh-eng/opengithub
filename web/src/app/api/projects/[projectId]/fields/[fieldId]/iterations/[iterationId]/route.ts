import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectIterationUpdateRequest,
  updateProjectIterationFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string; iterationId: string }>;
};

function parseRequest(input: unknown): ProjectIterationUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = typeof body.name === "string" ? body.name.trim() : "";
  const startDate = typeof body.startDate === "string" ? body.startDate : "";
  const durationDays =
    typeof body.durationDays === "number" ? body.durationDays : 0;
  if (!name || !startDate || durationDays < 1) return null;
  return { name, startDate, durationDays };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId, iterationId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Iteration name, start date, and duration are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const settings = await updateProjectIterationFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      decodeURIComponent(iterationId),
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
          code: "project_iteration_update_failed",
          message: "Project iteration could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
