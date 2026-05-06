import { type NextRequest, NextResponse } from "next/server";
import { getRepositoryWikiFromCookie } from "@/lib/api";

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

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, slug } = await context.params;
  const ownerLogin = decodePathSegment(owner);
  const repositoryName = decodePathSegment(repo);
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

  const result = await getRepositoryWikiFromCookie(
    request.headers.get("cookie"),
    ownerLogin,
    repositoryName,
    wikiSlug,
  );

  if (!result.ok) {
    return NextResponse.json(
      {
        error: {
          code: result.code ?? "wiki_unavailable",
          message: result.message,
        },
      },
      { status: result.status },
    );
  }

  if (result.wiki.state.kind !== "ready" || !result.wiki.page) {
    return NextResponse.json(
      {
        error: {
          code: result.wiki.state.kind,
          message: result.wiki.state.message,
        },
      },
      { status: 404 },
    );
  }

  return NextResponse.json({
    page: {
      id: result.wiki.page.id,
      title: result.wiki.page.title,
      slug: result.wiki.page.slug,
      href: result.wiki.page.href,
    },
    outline: result.wiki.page.outline,
  });
}
