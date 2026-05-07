import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateIssueRequest,
  createRepositoryIssueFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function malformedJsonError(message: string) {
  return NextResponse.json(
    {
      error: {
        code: "validation_failed",
        message,
      },
      status: 422,
      details: {
        field: "title",
        reason: message,
      },
    },
    { status: 422 },
  );
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  let body: CreateIssueRequest;

  try {
    body = (await request.json()) as CreateIssueRequest;
  } catch {
    return malformedJsonError("Issue creation payload must be valid JSON");
  }

  try {
    const issue = await createRepositoryIssueFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      {
        ...body,
        title: body.title?.trim() ?? "",
        body: body.body?.trim() ? body.body : null,
      },
    );
    return NextResponse.json(
      {
        ...issue,
        href: `/${decodeURIComponent(owner)}/${decodeURIComponent(repo)}/issues/${issue.number}`,
      },
      { status: 201 },
    );
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "issue_create_failed",
          message: "Issue could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
