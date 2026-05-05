import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type DiscussionCategoryOrderRequest,
  reorderRepositoryDiscussionCategoriesFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function parseOrder(input: unknown): DiscussionCategoryOrderRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const items = (input as Record<string, unknown>).items;
  if (!Array.isArray(items) || items.length === 0) return null;
  const parsed = items
    .map((item) => {
      if (!item || typeof item !== "object" || Array.isArray(item)) return null;
      const row = item as Record<string, unknown>;
      if (typeof row.id !== "string") return null;
      return {
        id: row.id,
        position: typeof row.position === "number" ? row.position : 1,
        sectionId: typeof row.sectionId === "string" ? row.sectionId : null,
      };
    })
    .filter((item): item is DiscussionCategoryOrderRequest["items"][number] =>
      Boolean(item),
    );
  return parsed.length ? { items: parsed } : null;
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const mutation = parseOrder(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Discussion category order payload is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await reorderRepositoryDiscussionCategoriesFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
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
          code: "repository_discussion_category_order_failed",
          message: "Discussion category order could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
