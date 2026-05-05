import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  addProjectItemFromCookie,
  type ProjectItemAddRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseAddRequest(input: unknown): ProjectItemAddRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const itemType = typeof body.itemType === "string" ? body.itemType : null;
  const title = typeof body.title === "string" ? body.title.trim() : null;
  const url = typeof body.url === "string" ? body.url.trim() : null;
  if (itemType === "draft_issue" && !title) return null;
  if ((itemType === "issue" || itemType === "pull_request") && !url) {
    return null;
  }
  return {
    itemType,
    title,
    body: typeof body.body === "string" ? body.body.trim() : null,
    url,
    issueId: typeof body.issueId === "string" ? body.issueId : null,
    pullRequestId:
      typeof body.pullRequestId === "string" ? body.pullRequestId : null,
    positionAfterItemId:
      typeof body.positionAfterItemId === "string"
        ? body.positionAfterItemId
        : null,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseAddRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Add a draft title or paste an issue or pull request URL.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const workspace = await addProjectItemFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      mutation,
    );
    return NextResponse.json(workspace, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_item_add_failed",
          message: "Project item could not be added.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
