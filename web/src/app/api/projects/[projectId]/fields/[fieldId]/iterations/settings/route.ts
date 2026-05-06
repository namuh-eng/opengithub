import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectIterationSettingsRequest,
  updateProjectIterationSettingsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
};

function parseRequest(input: unknown): ProjectIterationSettingsRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const startDate = typeof body.startDate === "string" ? body.startDate : "";
  const duration = typeof body.duration === "number" ? body.duration : 0;
  const durationUnit =
    body.durationUnit === "days" || body.durationUnit === "weeks"
      ? body.durationUnit
      : null;
  if (!startDate || !durationUnit || duration < 1) return null;
  return {
    startDate,
    duration,
    durationUnit,
    generatedIterations:
      typeof body.generatedIterations === "number"
        ? body.generatedIterations
        : null,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Start date, duration, and duration unit are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const settings = await updateProjectIterationSettingsFromCookie(
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
          code: "project_iteration_settings_failed",
          message: "Project iteration settings could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
