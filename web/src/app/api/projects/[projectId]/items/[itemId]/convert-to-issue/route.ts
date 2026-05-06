import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  convertProjectDraftToIssueFromCookie,
  type ProjectDraftConvertRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

function parseConvertRequest(
  input: unknown,
): ProjectDraftConvertRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  if (typeof body.repositoryId !== "string" || !body.repositoryId.trim()) {
    return null;
  }
  return {
    repositoryId: body.repositoryId,
    labelIds: Array.isArray(body.labelIds)
      ? body.labelIds.filter((id): id is string => typeof id === "string")
      : [],
    assigneeUserIds: Array.isArray(body.assigneeUserIds)
      ? body.assigneeUserIds.filter(
          (id): id is string => typeof id === "string",
        )
      : [],
    milestoneId: typeof body.milestoneId === "string" ? body.milestoneId : null,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  const mutation = parseConvertRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Choose a repository before converting this draft.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const detail = await convertProjectDraftToIssueFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
      mutation,
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_draft_convert_failed",
          message: "Draft project item could not be converted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
