import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CopyProjectRequest,
  copyProjectFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseCopyRequest(input: unknown): CopyProjectRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const title = typeof body.title === "string" ? body.title.trim() : "";
  if (!title) return null;
  return {
    title,
    includeDraftIssues: body.includeDraftIssues === true,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseCopyRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Project title is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const copied = await copyProjectFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      mutation,
    );
    return NextResponse.json(copied, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_copy_failed",
          message: "Project could not be copied.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
