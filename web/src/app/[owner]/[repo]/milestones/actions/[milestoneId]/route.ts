import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteRepositoryMilestoneFromCookie,
  type RepositoryMilestoneMutation,
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

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, milestoneId } = await context.params;
  const body = await request.json().catch(() => null);

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
