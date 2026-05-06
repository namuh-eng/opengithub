import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateProjectAccessFromCookie,
  type ProjectAccessMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

const roles = new Set(["read", "write", "admin"]);

function parseCreate(input: unknown): ProjectAccessMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const targetType = body.targetType;
  const role = typeof body.role === "string" ? body.role : "";
  const targetId =
    typeof body.targetId === "string" ? body.targetId.trim() : "";
  if ((targetType !== "user" && targetType !== "team") || !roles.has(role)) {
    return null;
  }
  if (!targetId) return null;
  const expectedUpdatedAt =
    typeof body.expectedUpdatedAt === "string" ? body.expectedUpdatedAt : null;
  return targetType === "user"
    ? {
        action: "add-user",
        userId: targetId,
        role: role as "read" | "write" | "admin",
        expectedUpdatedAt,
      }
    : {
        action: "add-team",
        teamId: targetId,
        role: role as "read" | "write" | "admin",
        expectedUpdatedAt,
      };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseCreate(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Target and role are required.",
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
