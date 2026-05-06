import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryLabelFromCookie,
  type RepositoryLabelMutationRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
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

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result = await createRepositoryLabelFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      labelRequest(body),
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_label_failed",
          message: "Repository label could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
