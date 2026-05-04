import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateOrganizationPeopleAdminFromCookie,
  type OrganizationInvitationMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ org: string }>;
};

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function parseMutation(input: unknown): OrganizationInvitationMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");

  if (action === "invite") {
    const emailOrLogin = stringField(body, "emailOrLogin");
    const role = stringField(body, "role");
    if (!emailOrLogin || (role !== "admin" && role !== "member")) {
      return null;
    }
    const teamIds = Array.isArray(body.teamIds)
      ? body.teamIds.filter(
          (value): value is string => typeof value === "string",
        )
      : [];
    return { action, emailOrLogin, role, teamIds };
  }
  if (action === "retry") {
    const invitationId = stringField(body, "invitationId");
    return invitationId ? { action, invitationId } : null;
  }
  if (action === "cancel") {
    const invitationId = stringField(body, "invitationId");
    return invitationId ? { action, invitationId } : null;
  }

  return null;
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { org } = await context.params;
  const mutation = parseMutation(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Organization people action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const people = await mutateOrganizationPeopleAdminFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(org),
      mutation,
    );
    return NextResponse.json(people);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "organization_people_failed",
          message: "Organization people update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
