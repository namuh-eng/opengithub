import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  bulkAddProjectItemsFromCookie,
  type ProjectItemAddRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseBulkRequest(
  input: unknown,
): { items: ProjectItemAddRequest[] } | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  if (!Array.isArray(body.items) || body.items.length === 0) return null;
  const items = body.items
    .filter(
      (item): item is Record<string, unknown> =>
        Boolean(item) && typeof item === "object",
    )
    .map((item) => ({
      itemType: typeof item.itemType === "string" ? item.itemType : "issue",
      title: typeof item.title === "string" ? item.title.trim() : null,
      body: typeof item.body === "string" ? item.body.trim() : null,
      url: typeof item.url === "string" ? item.url.trim() : null,
      issueId: typeof item.issueId === "string" ? item.issueId : null,
      pullRequestId:
        typeof item.pullRequestId === "string" ? item.pullRequestId : null,
      positionAfterItemId:
        typeof item.positionAfterItemId === "string"
          ? item.positionAfterItemId
          : null,
    }))
    .filter(
      (item) => item.url || item.issueId || item.pullRequestId || item.title,
    );
  return items.length ? { items } : null;
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseBulkRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message:
            "Bulk add requires at least one issue, pull request, or draft item.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const workspace = await bulkAddProjectItemsFromCookie(
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
          code: "project_item_bulk_add_failed",
          message: "Project items could not be added.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
