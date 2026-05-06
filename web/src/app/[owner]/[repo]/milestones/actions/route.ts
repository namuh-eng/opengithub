import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryMilestoneFromCookie,
  type RepositoryMilestoneMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
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

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result = await createRepositoryMilestoneFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      milestoneRequest(body),
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_milestone_failed",
          message: "Repository milestone could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
