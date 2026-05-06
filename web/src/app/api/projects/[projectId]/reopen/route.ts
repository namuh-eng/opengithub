import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectLifecycleRequest,
  reopenProjectFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseRequest(input: unknown): ProjectLifecycleRequest {
  const body =
    input && typeof input === "object" && !Array.isArray(input)
      ? (input as Record<string, unknown>)
      : {};
  return {
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  try {
    const settings = await reopenProjectFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      parseRequest(await request.json().catch(() => null)),
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_reopen_failed",
          message: "Project could not be reopened.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
