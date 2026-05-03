import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryPagesSettingsFromCookie,
  type RepositoryPagesMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const actions = new Set([
  "update-source",
  "save-domain",
  "remove-domain",
  "recheck-dns",
  "update-https",
  "request-deployment",
  "unpublish-pages",
]);
const sourceKinds = new Set(["none", "branch", "actions"]);

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function optionalStringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseMutation(input: unknown): RepositoryPagesMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");
  if (!actions.has(action)) return null;

  if (action === "update-source") {
    const kind = stringField(body, "kind");
    if (!sourceKinds.has(kind)) return null;
    return {
      action,
      branch: optionalStringField(body, "branch"),
      folder: optionalStringField(body, "folder"),
      kind: kind as "none" | "branch" | "actions",
      workflowArtifactName: optionalStringField(body, "workflowArtifactName"),
      workflowId: optionalStringField(body, "workflowId"),
    };
  }

  if (action === "save-domain") {
    const domain = stringField(body, "domain");
    return domain ? { action, domain } : null;
  }

  if (action === "update-https") {
    return typeof body.enforced === "boolean"
      ? { action, enforced: body.enforced }
      : null;
  }

  return { action } as RepositoryPagesMutation;
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const mutation = parseMutation(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Repository Pages action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await mutateRepositoryPagesSettingsFromCookie(
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
          code: "repository_pages_failed",
          message: "Repository Pages update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
