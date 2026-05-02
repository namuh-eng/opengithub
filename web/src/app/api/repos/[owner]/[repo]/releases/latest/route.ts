import { getRepositoryFromCookie, getSessionFromCookie } from "@/lib/api";
import { getRepositoryLatestRelease } from "@/lib/releases";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
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

  const latest = getRepositoryLatestRelease(repository, session);
  if (!latest) {
    return Response.json(
      {
        error: { code: "not_found", message: "No stable release found." },
        status: 404,
      },
      { status: 404 },
    );
  }

  return Response.json(latest);
}
