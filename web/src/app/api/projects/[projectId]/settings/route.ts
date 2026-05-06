import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectSettingsUpdateRequest,
  updateProjectSettingsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseRequest(input: unknown): ProjectSettingsUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const title = typeof body.title === "string" ? body.title.trim() : "";
  if (!title) return null;
  return {
    title,
    description:
      typeof body.description === "string" ? body.description.trim() : null,
    readme: typeof body.readme === "string" ? body.readme : null,
    visibility:
      typeof body.visibility === "string" ? body.visibility.trim() : null,
    defaultRepositoryId:
      typeof body.defaultRepositoryId === "string" &&
      body.defaultRepositoryId.trim()
        ? body.defaultRepositoryId.trim()
        : null,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Project title is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateProjectSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_settings_update_failed",
          message: "Project settings could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
