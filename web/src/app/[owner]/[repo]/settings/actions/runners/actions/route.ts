import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryActionsRunnerSettingsFromCookie,
  type RepositoryActionsRunnerMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

function parseMutation(input: unknown): RepositoryActionsRunnerMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = typeof body.action === "string" ? body.action : "";
  if (action === "create-runner") {
    const name = typeof body.name === "string" ? body.name.trim() : "";
    const labels = Array.isArray(body.labels)
      ? body.labels.filter(
          (label): label is string => typeof label === "string",
        )
      : [];
    return name && labels.length ? { action, labels, name } : null;
  }
  if (action === "update-settings") {
    const concurrencyLimit = Number(body.concurrencyLimit);
    return Number.isInteger(concurrencyLimit)
      ? {
          action,
          cancelInProgress: body.cancelInProgress === true,
          concurrencyLimit,
        }
      : null;
  }
  if (action === "schedule-jobs") {
    return { action };
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
          message: "Repository Actions runner action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const result = await mutateRepositoryActionsRunnerSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      mutation,
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_actions_runners_failed",
          message: "Repository Actions runner update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
