import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  saveRepositoryForkDefaultsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const periods = new Set(["24h", "3d", "1w", "1m", "all"]);
const repositoryTypes = new Set([
  "all",
  "active",
  "inactive",
  "archived",
  "starred",
]);
const sorts = new Set([
  "most_starred",
  "recently_pushed",
  "recently_created",
  "recently_updated",
  "name",
]);

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const body = (await request.json().catch(() => null)) as Record<
    string,
    unknown
  > | null;
  const period = body ? stringField(body, "period") : "";
  const repositoryType = body ? stringField(body, "repositoryType") : "";
  const sort = body ? stringField(body, "sort") : "";

  if (
    !periods.has(period) ||
    !repositoryTypes.has(repositoryType) ||
    !sorts.has(sort)
  ) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Repository fork defaults are invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const forks = await saveRepositoryForkDefaultsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      { period, repositoryType, sort },
    );
    return NextResponse.json(forks);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_fork_defaults_failed",
          message: "Repository fork defaults failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
