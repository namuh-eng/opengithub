import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type UpdateRepositorySettingsRequest,
  updateRepositorySettingsFromCookie,
} from "@/lib/api";

const VISIBILITIES = new Set(["public", "private", "internal"]);

function validationError(message: string) {
  return NextResponse.json(
    {
      error: { code: "validation_failed", message },
      status: 422,
    },
    { status: 422 },
  );
}

export async function PATCH(
  request: NextRequest,
  context: { params: Promise<{ owner: string; repo: string }> },
) {
  let body: UpdateRepositorySettingsRequest;
  try {
    body = (await request.json()) as UpdateRepositorySettingsRequest;
  } catch {
    return validationError("Repository settings payload must be valid JSON");
  }

  if (body.visibility && !VISIBILITIES.has(body.visibility)) {
    return validationError("Visibility must be public, private, or internal");
  }
  if (
    body.mergeMethods &&
    !body.mergeMethods.mergeCommit &&
    !body.mergeMethods.squash &&
    !body.mergeMethods.rebase
  ) {
    return validationError(
      "At least one pull request merge method must stay enabled",
    );
  }

  const { owner, repo } = await context.params;
  try {
    const settings = await updateRepositorySettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      body,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "settings_update_failed",
          message: "Repository settings could not be updated",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
