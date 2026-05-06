import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectTemplateUpdateRequest,
  updateProjectTemplateFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseRequest(input: unknown): ProjectTemplateUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  if (typeof body.isTemplate !== "boolean") return null;
  return {
    isTemplate: body.isTemplate,
    title: typeof body.title === "string" ? body.title.trim() : null,
    description:
      typeof body.description === "string" ? body.description.trim() : null,
    isPublic: typeof body.isPublic === "boolean" ? body.isPublic : false,
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
          message: "Template settings payload is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateProjectTemplateFromCookie(
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
          code: "project_template_update_failed",
          message: "Project template settings could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
