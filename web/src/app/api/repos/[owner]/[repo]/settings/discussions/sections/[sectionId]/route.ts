import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteRepositoryDiscussionCategorySectionFromCookie,
  type UpdateDiscussionCategorySectionRequest,
  updateRepositoryDiscussionCategorySectionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; sectionId: string }>;
};

function parseSection(
  input: unknown,
): UpdateDiscussionCategorySectionRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const name = (input as Record<string, unknown>).name;
  if (typeof name !== "string" || !name.trim()) return null;
  return { name: name.trim() };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, sectionId } = await context.params;
  const mutation = parseSection(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Discussion category section update is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateRepositoryDiscussionCategorySectionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(sectionId),
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
          code: "repository_discussion_category_section_failed",
          message: "Discussion category section could not be updated.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, sectionId } = await context.params;
  try {
    const settings = await deleteRepositoryDiscussionCategorySectionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(sectionId),
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_discussion_category_section_failed",
          message: "Discussion category section could not be deleted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
