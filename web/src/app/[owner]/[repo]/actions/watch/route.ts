import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  getRepositoryWatchSettingsFromCookie,
  setRepositoryWatchFromCookie,
  updateRepositoryWatchSettingsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

function watchErrorResponse(error: unknown) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "repository_watch_failed",
        message: "Repository watch update failed",
      },
      status: 502,
    },
    { status: envelope?.status ?? 502 },
  );
}

async function updateWatch(
  request: NextRequest,
  context: RouteContext,
  watching: boolean,
) {
  const { owner, repo } = await context.params;
  try {
    const social = await setRepositoryWatchFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      watching,
    );
    return NextResponse.json(social);
  } catch (error) {
    return watchErrorResponse(error);
  }
}

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  try {
    const settings = await getRepositoryWatchSettingsFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
    );
    return NextResponse.json(settings);
  } catch (error) {
    return watchErrorResponse(error);
  }
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  try {
    const patch = await request.json();
    const settings = await updateRepositoryWatchSettingsFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      patch,
    );
    return NextResponse.json(settings);
  } catch (error) {
    return watchErrorResponse(error);
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateWatch(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateWatch(request, context, false);
}
