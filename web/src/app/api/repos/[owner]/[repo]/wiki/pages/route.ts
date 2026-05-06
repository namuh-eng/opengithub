import { type NextRequest, NextResponse } from "next/server";
import { createRepositoryWikiPageFromCookie } from "@/lib/api";

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
    const result = await createRepositoryWikiPageFromCookie(
      request.headers.get("cookie"),
      decodePathSegment(owner),
      decodePathSegment(repo),
      body,
    );
    return NextResponse.json(result);
  } catch (error) {
    return NextResponse.json(
      {
        error: {
          code: "wiki_page_save_failed",
          message:
            error instanceof Error
              ? error.message
              : "Repository wiki page save failed.",
        },
      },
      { status: 400 },
    );
  }
}
