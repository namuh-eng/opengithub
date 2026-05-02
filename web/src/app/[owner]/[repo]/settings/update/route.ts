import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type RepositorySettingsPatch,
  updateRepositorySettingsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const patch = (await request
    .json()
    .catch(() => ({}))) as RepositorySettingsPatch;

  try {
    const settings = await updateRepositorySettingsFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      patch,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_settings_failed",
          message: "Repository settings failed to save",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
