import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectFromCookie,
  type ProjectLifecycleRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseRequest(input: unknown): ProjectLifecycleRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const confirmation =
    typeof body.confirmation === "string" ? body.confirmation.trim() : "";
  if (!confirmation) return null;
  return {
    confirmation,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const lifecycleRequest = parseRequest(await request.json().catch(() => null));
  if (!lifecycleRequest) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Type the project title to confirm deletion.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const response = await deleteProjectFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      lifecycleRequest,
    );
    return NextResponse.json(response);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_delete_failed",
          message: "Project could not be deleted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
