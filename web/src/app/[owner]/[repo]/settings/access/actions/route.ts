import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryAccessSettingsFromCookie,
  type RepositoryAccessMutation,
  type RepositoryAccessRole,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const writableRoles = new Set(["read", "triage", "write", "maintain", "admin"]);
type WritableAccessRole = Exclude<RepositoryAccessRole, "owner">;

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function roleField(input: Record<string, unknown>) {
  const role = stringField(input, "role");
  return writableRoles.has(role) ? (role as WritableAccessRole) : null;
}

function parseMutation(input: unknown): RepositoryAccessMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");
  const role = roleField(body);

  if (action === "invite-person" && role) {
    const emailOrLogin = stringField(body, "emailOrLogin");
    return emailOrLogin ? { action, emailOrLogin, role } : null;
  }
  if (action === "grant-team" && role) {
    const teamSlug = stringField(body, "teamSlug");
    return teamSlug ? { action, teamSlug, role } : null;
  }
  if (action === "update-person-role" && role) {
    const userId = stringField(body, "userId");
    return userId ? { action, userId, role } : null;
  }
  if (action === "update-team-role" && role) {
    const teamId = stringField(body, "teamId");
    return teamId ? { action, teamId, role } : null;
  }
  if (action === "remove-person") {
    const userId = stringField(body, "userId");
    return userId ? { action, userId } : null;
  }
  if (action === "remove-team") {
    const teamId = stringField(body, "teamId");
    return teamId ? { action, teamId } : null;
  }
  if (action === "cancel-invitation") {
    const invitationId = stringField(body, "invitationId");
    return invitationId ? { action, invitationId } : null;
  }

  return null;
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const mutation = parseMutation(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Repository access action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await mutateRepositoryAccessSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
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
          code: "repository_access_failed",
          message: "Repository access update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
