import { cookies } from "next/headers";
import {
  createRepositoryReleaseAssetFromCookie,
  createRepositoryReleaseFromCookie,
  deleteRepositoryReleaseAssetFromCookie,
  deleteRepositoryReleaseFromCookie,
  publishRepositoryReleaseFromCookie,
  type ReleaseAssetMutation,
  type ReleaseMutation,
  updateRepositoryReleaseFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

export async function POST(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const cookie = (await cookies()).toString();
  const payload = await request.json().catch(() => null);
  const action = String(payload?.action ?? "");

  try {
    if (action === "create") {
      const release = await createRepositoryReleaseFromCookie(
        cookie,
        owner,
        repo,
        payload?.release as ReleaseMutation,
      );
      return Response.json(release, { status: 201 });
    }
    if (action === "update") {
      const release = await updateRepositoryReleaseFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.releaseId ?? ""),
        payload?.release as ReleaseMutation,
      );
      return Response.json(release);
    }
    if (action === "publish") {
      const release = await publishRepositoryReleaseFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.releaseId ?? ""),
      );
      return Response.json(release);
    }
    if (action === "delete") {
      await deleteRepositoryReleaseFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.releaseId ?? ""),
      );
      return Response.json({ ok: true });
    }
    if (action === "createAsset") {
      const release = await createRepositoryReleaseAssetFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.releaseId ?? ""),
        payload?.asset as ReleaseAssetMutation,
      );
      return Response.json(release, { status: 201 });
    }
    if (action === "deleteAsset") {
      const release = await deleteRepositoryReleaseAssetFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.releaseId ?? ""),
        String(payload?.assetId ?? ""),
      );
      return Response.json(release);
    }
    return Response.json(
      {
        error: {
          code: "validation_failed",
          message: "Unknown release action.",
        },
      },
      { status: 422 },
    );
  } catch (error) {
    const cause = error instanceof Error ? error.cause : null;
    const envelope =
      cause && typeof cause === "object" && "error" in cause
        ? cause
        : {
            error: {
              code: "release_action_failed",
              message:
                error instanceof Error
                  ? error.message
                  : "Release action could not be completed.",
            },
            status: 500,
          };
    const status =
      typeof envelope === "object" && envelope && "status" in envelope
        ? Number(envelope.status) || 500
        : 500;
    return Response.json(envelope, { status });
  }
}
