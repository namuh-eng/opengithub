import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  linkProjectRepositoryFromCookie,
  type ProjectRepositoryLinkRequest,
  unlinkProjectRepositoryFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; repositoryId: string }>;
};

function parseRequest(input: unknown): ProjectRepositoryLinkRequest {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return { expectedUpdatedAt: null };
  }
  const body = input as Record<string, unknown>;
  return {
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

function fallbackEnvelope(message: string) {
  return {
    error: { code: "project_repository_link_failed", message },
    status: 502,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId, repositoryId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  try {
    const settings = await linkProjectRepositoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(repositoryId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? fallbackEnvelope("Project repository could not be linked."),
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, repositoryId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  try {
    const settings = await unlinkProjectRepositoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(repositoryId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? fallbackEnvelope("Project repository could not be removed."),
      { status: envelope?.status ?? 502 },
    );
  }
}
