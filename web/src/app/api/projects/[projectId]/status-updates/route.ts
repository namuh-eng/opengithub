import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectStatusUpdateFromCookie,
  type ProjectStatusUpdateRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseDate(value: unknown) {
  return typeof value === "string" && value ? value : null;
}

function parseRequest(input: unknown): ProjectStatusUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const status = typeof body.status === "string" ? body.status.trim() : "";
  if (!status) return null;
  return {
    status,
    body: typeof body.body === "string" ? body.body.trim() : null,
    startDate: parseDate(body.startDate),
    targetDate: parseDate(body.targetDate),
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Project status is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await createProjectStatusUpdateFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
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
          code: "project_status_update_failed",
          message: "Project status update could not be published.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
