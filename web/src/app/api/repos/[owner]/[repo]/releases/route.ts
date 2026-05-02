import { getRepositoryFromCookie, getSessionFromCookie } from "@/lib/api";
import { getRepositoryReleasesView } from "@/lib/releases";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function parsePage(value: string | null) {
  const page = Number(value);
  return Number.isFinite(page) && page > 0 ? Math.floor(page) : 1;
}

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const url = new URL(request.url);
  const cookie = request.headers.get("cookie");
  const [session, repository] = await Promise.all([
    getSessionFromCookie(cookie),
    getRepositoryFromCookie(cookie, ownerLogin, repositoryName),
  ]);

  if (!repository) {
    return Response.json(
      {
        error: { code: "not_found", message: "Repository not found." },
        status: 404,
      },
      { status: 404 },
    );
  }

  return Response.json(
    getRepositoryReleasesView(repository, session, {
      page: parsePage(url.searchParams.get("page")),
      pageSize: parsePage(url.searchParams.get("pageSize")),
    }),
  );
}
