import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectIterationFromCookie,
  type ProjectIterationCreateRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string }>;
};

function parseRequest(input: unknown): ProjectIterationCreateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return {};
  const body = input as Record<string, unknown>;
  return {
    name: typeof body.name === "string" ? body.name : null,
    startDate: typeof body.startDate === "string" ? body.startDate : null,
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
        error: { code: "validation_failed", message: "Invalid iteration." },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const settings = await createProjectIterationFromCookie(
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
          code: "project_iteration_create_failed",
          message: "Project iteration could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
