import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryActionsSecretsSettingsFromCookie,
  type RepositoryActionsSecretsMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const actions = new Set([
  "create-secret",
  "update-secret",
  "delete-secret",
  "create-variable",
  "update-variable",
  "delete-variable",
]);

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function rawStringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value : "";
}

function parseMutation(
  input: unknown,
): RepositoryActionsSecretsMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }

  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");
  if (!actions.has(action)) return null;

  if (action === "create-secret" || action === "create-variable") {
    const name = stringField(body, "name");
    const value = rawStringField(body, "value");
    return name && value.trim() ? { action, name, value } : null;
  }

  if (action === "update-secret" || action === "update-variable") {
    const currentName = stringField(body, "currentName");
    const name = stringField(body, "name");
    const value = rawStringField(body, "value");
    return currentName && name && value.trim()
      ? { action, currentName, name, value }
      : null;
  }

  if (action === "delete-secret" || action === "delete-variable") {
    const name = stringField(body, "name");
    return name ? { action, name } : null;
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
          message: "Repository Actions secrets action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await mutateRepositoryActionsSecretsSettingsFromCookie(
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
          code: "repository_actions_secrets_failed",
          message: "Repository Actions secrets update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
