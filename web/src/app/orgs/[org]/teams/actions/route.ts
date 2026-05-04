import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateOrganizationTeamRequest,
  createOrganizationTeamFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ org: string }>;
};

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function parseCreateTeam(input: unknown): CreateOrganizationTeamRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const name = stringField(body, "name");
  const visibility = stringField(body, "visibility");
  if (!name || (visibility !== "visible" && visibility !== "secret")) {
    return null;
  }

  const description = stringField(body, "description");
  const parentTeamId = stringField(body, "parentTeamId");
  return {
    name,
    description: description || null,
    parentTeamId: parentTeamId || null,
    visibility,
    notificationsEnabled: body.notificationsEnabled !== false,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { org } = await context.params;
  const input = parseCreateTeam(await request.json().catch(() => null));
  if (!input) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Team creation input is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const result = await createOrganizationTeamFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(org),
      input,
    );
    return NextResponse.json(result, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "organization_team_failed",
          message: "Team creation failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
