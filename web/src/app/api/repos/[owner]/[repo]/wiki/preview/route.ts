import { type NextRequest, NextResponse } from "next/server";
import { previewRepositoryWikiFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function decodePathSegment(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const preview = await previewRepositoryWikiFromCookie(
      request.headers.get("cookie"),
      decodePathSegment(owner),
      decodePathSegment(repo),
      body,
    );
    return NextResponse.json(preview);
  } catch (error) {
    const envelope = (error instanceof Error ? error.cause : null) as {
      error: { code: string; message: string };
      status: number;
    } | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "wiki_preview_failed",
          message:
            error instanceof Error
              ? error.message
              : "Repository wiki preview failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
