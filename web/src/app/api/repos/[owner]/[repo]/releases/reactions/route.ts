import { getRepositoryFromCookie, getSessionFromCookie } from "@/lib/api";
import {
  type ReleaseReactionKind,
  toggleReleaseReaction,
} from "@/lib/releases";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

const ALLOWED_KINDS = new Set(["heart", "hooray", "rocket", "eyes"]);

export async function POST(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const cookie = request.headers.get("cookie");
  const session = await getSessionFromCookie(cookie);
  if (!session.authenticated) {
    return Response.json(
      {
        error: {
          code: "unauthorized",
          message: "Sign in to react to releases.",
        },
        status: 401,
      },
      { status: 401 },
    );
  }

  const body = (await request.json().catch(() => null)) as {
    releaseId?: string;
    kind?: string;
  } | null;
  if (!body?.releaseId || !body.kind || !ALLOWED_KINDS.has(body.kind)) {
    return Response.json(
      {
        error: {
          code: "bad_request",
          message: "releaseId and a valid reaction kind are required.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  const repository = await getRepositoryFromCookie(
    cookie,
    decodeURIComponent(owner),
    decodeURIComponent(repo),
  );
  if (!repository) {
    return Response.json(
      {
        error: { code: "not_found", message: "Repository not found." },
        status: 404,
      },
      { status: 404 },
    );
  }

  const result = toggleReleaseReaction(
    repository,
    session,
    body.releaseId,
    body.kind as ReleaseReactionKind,
  );
  if (!result) {
    return Response.json(
      {
        error: { code: "not_found", message: "Release not found." },
        status: 404,
      },
      { status: 404 },
    );
  }

  return Response.json(result);
}
