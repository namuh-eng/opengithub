import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateProjectAccessFromCookie,
  type ProjectAccessMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; grantId: string }>;
};

const roles = new Set(["read", "write", "admin"]);

function parsePatch(
  input: unknown,
  grantId: string,
): ProjectAccessMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const role = typeof body.role === "string" ? body.role : "";
  if (!roles.has(role)) return null;
  return {
    action: "update-grant",
    grantId,
    role: role as "read" | "write" | "admin",
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

function parseDelete(input: unknown, grantId: string): ProjectAccessMutation {
  const body =
    input && typeof input === "object" && !Array.isArray(input)
      ? (input as Record<string, unknown>)
      : {};
  return {
    action: "remove-grant",
    grantId,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

async function runMutation(
  request: NextRequest,
  projectId: string,
  mutation: ProjectAccessMutation | null,
) {
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "A valid project access role is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const settings = await mutateProjectAccessFromCookie(
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
          code: "project_access_update_failed",
          message: "Project access could not be changed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, grantId } = await context.params;
  return runMutation(
    request,
    projectId,
    parsePatch(
      await request.json().catch(() => null),
      decodeURIComponent(grantId),
    ),
  );
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, grantId } = await context.params;
  return runMutation(
    request,
    projectId,
    parseDelete(
      await request.json().catch(() => null),
      decodeURIComponent(grantId),
    ),
  );
}
