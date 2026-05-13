import { type NextRequest, NextResponse } from "next/server";
import { updateRepositoryWikiPageFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; slug: string[] }>;
};

function decodePathSegment(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, slug } = await context.params;
  const wikiSlug = slug.map(decodePathSegment).filter(Boolean).join("/");

  if (
    !wikiSlug ||
    wikiSlug.split("/").some((segment) => segment === "." || segment === "..")
  ) {
    return NextResponse.json(
      {
        error: {
          code: "invalid_wiki_slug",
          message: "Wiki page slug is invalid.",
        },
      },
      { status: 400 },
    );
  }

  const body = await request.json().catch(() => null);

  try {
    const result = await updateRepositoryWikiPageFromCookie(
      request.headers.get("cookie"),
      decodePathSegment(owner),
      decodePathSegment(repo),
      wikiSlug,
      body,
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (error instanceof Error ? error.cause : null) as {
      error: { code: string; message: string };
      status: number;
    } | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "wiki_page_save_failed",
          message:
            error instanceof Error
              ? error.message
              : "Repository wiki page failed to save.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
