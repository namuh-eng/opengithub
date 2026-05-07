import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, forkRepositoryFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  try {
    const payload = (await request.json().catch(() => ({}))) as {
      destinationOwner?: string;
      name?: string;
      mainBranchOnly?: boolean;
    };
    const fork = await forkRepositoryFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      payload,
    );
    return NextResponse.json(fork, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_fork_failed",
          message: "Repository fork failed",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
