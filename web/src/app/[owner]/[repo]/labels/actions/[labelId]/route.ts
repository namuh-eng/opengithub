import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteRepositoryLabelFromCookie,
  type RepositoryLabelMutationRequest,
  updateRepositoryLabelFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; labelId: string }>;
};

function labelRequest(body: unknown): RepositoryLabelMutationRequest {
  return {
    name:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { name?: unknown }).name === "string"
        ? (body as { name: string }).name
        : "",
    color:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { color?: unknown }).color === "string"
        ? (body as { color: string }).color
        : "",
    description:
      typeof body === "object" &&
      body !== null &&
      typeof (body as { description?: unknown }).description === "string"
        ? (body as { description: string }).description
        : null,
  };
}

function errorResponse(error: unknown, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: { code: "repository_label_failed", message },
      status: 500,
    },
    { status: envelope?.status ?? 500 },
  );
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, labelId } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result = await updateRepositoryLabelFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(labelId),
      labelRequest(body),
    );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Repository label could not be updated.");
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, labelId } = await context.params;

  try {
    const result = await deleteRepositoryLabelFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(labelId),
    );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Repository label could not be deleted.");
  }
}
