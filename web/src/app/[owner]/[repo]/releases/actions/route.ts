import { cookies } from "next/headers";
import {
  cancelRepositoryReleaseUploadIntentFromCookie,
  completeRepositoryReleaseUploadIntentFromCookie,
  createRepositoryReleaseAssetFromCookie,
  createRepositoryReleaseFromCookie,
  createRepositoryReleaseUploadIntentFromCookie,
  deleteRepositoryReleaseAssetFromCookie,
  deleteRepositoryReleaseFromCookie,
  type GeneratedReleaseNotesRequest,
  generateRepositoryReleaseNotesFromCookie,
  publishRepositoryReleaseFromCookie,
  type ReleaseAssetMutation,
  type ReleaseMutation,
  type ReleaseUploadCompleteRequest,
  type ReleaseUploadIntentRequest,
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
    if (action === "generatedNotes") {
      const preview = await generateRepositoryReleaseNotesFromCookie(
        cookie,
        owner,
        repo,
        payload?.request as GeneratedReleaseNotesRequest,
      );
      return Response.json(preview);
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
        { deleteTag: Boolean(payload?.deleteTag) },
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
    if (action === "createUploadIntent") {
      const intent = await createRepositoryReleaseUploadIntentFromCookie(
        cookie,
        owner,
        repo,
        payload?.asset as ReleaseUploadIntentRequest,
      );
      return Response.json(intent, { status: 201 });
    }
    if (action === "completeUploadIntent") {
      const release = await completeRepositoryReleaseUploadIntentFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.intentId ?? ""),
        payload?.completion as ReleaseUploadCompleteRequest,
      );
      return Response.json(release);
    }
    if (action === "cancelUploadIntent") {
      const intent = await cancelRepositoryReleaseUploadIntentFromCookie(
        cookie,
        owner,
        repo,
        String(payload?.intentId ?? ""),
        String(payload?.reason ?? "cancelled by user"),
      );
      return Response.json(intent);
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
