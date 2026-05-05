import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositorySecurityPolicyFromCookie,
  type RepositorySecurityPolicyMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function optionalStringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseMutation(
  input: unknown,
): RepositorySecurityPolicyMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const markdown = stringField(body, "markdown");
  const commitMessage = stringField(body, "commitMessage");
  if (!markdown || !commitMessage) {
    return null;
  }
  return {
    commitMessage,
    expectedContentSha: optionalStringField(body, "expectedContentSha"),
    markdown,
    path: optionalStringField(body, "path") ?? "SECURITY.md",
    ref: optionalStringField(body, "ref"),
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const input = await request.json().catch(() => null);
  const mutation = parseMutation(input);
  const action = String((input as { action?: unknown } | null)?.action ?? "");
  if (!mutation || (action !== "create" && action !== "update")) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Repository Security policy action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const policy = await mutateRepositorySecurityPolicyFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      mutation,
      action === "create" ? "POST" : "PATCH",
    );
    return NextResponse.json(policy, {
      status: action === "create" ? 201 : 200,
    });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "security_policy_update_failed",
          message: "Repository Security policy update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
