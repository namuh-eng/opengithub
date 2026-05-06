import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  closeRepositoryMilestoneFromCookie,
  deleteRepositoryMilestoneFromCookie,
  type RepositoryMilestoneMutation,
  type RepositoryMilestoneOrderRequest,
  reopenRepositoryMilestoneFromCookie,
  reorderRepositoryMilestoneItemsFromCookie,
  updateRepositoryMilestoneFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; milestoneId: string }>;
};

function milestoneRequest(body: unknown): RepositoryMilestoneMutation {
  return {
    title:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { title?: unknown }).title === "string"
        ? (body as { title: string }).title
        : "",
    description:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { description?: unknown }).description === "string"
        ? (body as { description: string }).description
        : null,
    dueOn:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { dueOn?: unknown }).dueOn === "string"
        ? (body as { dueOn: string }).dueOn
        : null,
  };
}

function errorResponse(error: unknown, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: { code: "repository_milestone_failed", message },
      status: 500,
    },
    { status: envelope?.status ?? 500 },
  );
}

function orderRequest(body: unknown): RepositoryMilestoneOrderRequest {
  const itemIds =
    typeof body === "object" &&
    body !== null &&
    Array.isArray((body as { itemIds?: unknown }).itemIds)
      ? (body as { itemIds: unknown[] }).itemIds.filter(
          (value): value is string => typeof value === "string",
        )
      : [];
  const expectedVersion =
    typeof body === "object" &&
    body !== null &&
    typeof (body as { expectedVersion?: unknown }).expectedVersion === "string"
      ? (body as { expectedVersion: string }).expectedVersion
      : null;
  return { itemIds, expectedVersion };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, milestoneId } = await context.params;
  const body = await request.json().catch(() => null);
  if (
    typeof body === "object" &&
    body !== null &&
    Array.isArray((body as { itemIds?: unknown }).itemIds)
  ) {
    try {
      const result = await reorderRepositoryMilestoneItemsFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(milestoneId),
        orderRequest(body),
      );
      return NextResponse.json(result);
    } catch (error) {
      return errorResponse(
        error,
        "Repository milestone order could not be saved.",
      );
    }
  }

  try {
    const result = await updateRepositoryMilestoneFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(milestoneId),
      milestoneRequest(body),
    );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Repository milestone could not be updated.");
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, milestoneId } = await context.params;

  try {
    await deleteRepositoryMilestoneFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(milestoneId),
    );
    return NextResponse.json({ ok: true });
  } catch (error) {
    return errorResponse(error, "Repository milestone could not be deleted.");
  }
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, milestoneId } = await context.params;
  const body = await request.json().catch(() => null);
  const action =
    typeof body === "object" &&
    body !== null &&
    typeof (body as { action?: unknown }).action === "string"
      ? (body as { action: string }).action
      : "";

  try {
    const result =
      action === "close"
        ? await closeRepositoryMilestoneFromCookie(
            request.headers.get("cookie"),
            decodeURIComponent(owner),
            decodeURIComponent(repo),
            decodeURIComponent(milestoneId),
          )
        : action === "reopen"
          ? await reopenRepositoryMilestoneFromCookie(
              request.headers.get("cookie"),
              decodeURIComponent(owner),
              decodeURIComponent(repo),
              decodeURIComponent(milestoneId),
            )
          : null;
    if (!result) {
      return NextResponse.json(
        {
          error: {
            code: "validation_failed",
            message: "Milestone action must be close or reopen.",
          },
          status: 422,
        },
        { status: 422 },
      );
    }
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Repository milestone state could not change.");
  }
}
