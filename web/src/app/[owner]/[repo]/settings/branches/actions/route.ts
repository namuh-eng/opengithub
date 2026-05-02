import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type BranchPolicyEnforcement,
  mutateRepositoryBranchSettingsFromCookie,
  type RepositoryBranchPolicyMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const actions = new Set([
  "create-rule",
  "update-rule",
  "delete-rule",
  "create-ruleset",
  "update-ruleset",
  "delete-ruleset",
]);
const enforcements = new Set(["active", "evaluate", "disabled"]);
const requirementBooleanFields = [
  "requiresUpToDateBranch",
  "requiresConversationResolution",
  "requiresSignedCommits",
  "requiresLinearHistory",
  "requiresMergeQueue",
  "requiresDeployments",
  "locked",
  "restrictsPushes",
  "allowsForcePushes",
  "allowsDeletions",
] as const;

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function stringListField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

function bypassActorsField(input: Record<string, unknown>) {
  const value = input.bypassActors;
  if (!Array.isArray(value)) return [];
  return value
    .filter((item): item is Record<string, unknown> => {
      return !!item && typeof item === "object" && !Array.isArray(item);
    })
    .map((item) => ({
      actorId: stringField(item, "actorId"),
      actorType: stringField(item, "actorType"),
      label: stringField(item, "label"),
    }))
    .filter((item) => item.actorId && item.actorType && item.label);
}

function addRequirements(
  source: Record<string, unknown>,
  target: Record<string, unknown>,
) {
  const reviewCount = Number(source.requiredApprovingReviewCount ?? 0);
  target.requiredApprovingReviewCount = Number.isFinite(reviewCount)
    ? reviewCount
    : 0;
  target.requiredStatusChecks = stringListField(source, "requiredStatusChecks");
  target.requiredDeploymentEnvironments = stringListField(
    source,
    "requiredDeploymentEnvironments",
  );
  for (const field of requirementBooleanFields) {
    target[field] = source[field] === true;
  }
  target.bypassActors = bypassActorsField(source);
}

function parseMutation(input: unknown): RepositoryBranchPolicyMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");
  if (!actions.has(action)) return null;

  if (action === "delete-rule") {
    const ruleId = stringField(body, "ruleId");
    return ruleId ? { action, ruleId } : null;
  }
  if (action === "delete-ruleset") {
    const rulesetId = stringField(body, "rulesetId");
    return rulesetId ? { action, rulesetId } : null;
  }

  const enforcement = stringField(body, "enforcement");
  const base: Record<string, unknown> = {
    enforcement: enforcements.has(enforcement)
      ? (enforcement as BranchPolicyEnforcement)
      : "active",
  };
  addRequirements(body, base);

  if (action === "create-rule" || action === "update-rule") {
    const pattern = stringField(body, "pattern");
    if (!pattern) return null;
    const mutation = {
      ...base,
      action,
      description: stringField(body, "description") || null,
      pattern,
    } as const;
    if (action === "update-rule") {
      const ruleId = stringField(body, "ruleId");
      return ruleId ? { ...mutation, ruleId } : null;
    }
    return mutation;
  }

  if (action === "create-ruleset" || action === "update-ruleset") {
    const name = stringField(body, "name");
    const patterns = stringListField(body, "patterns");
    if (!name || patterns.length === 0) return null;
    const mutation = { ...base, action, name, patterns } as const;
    if (action === "update-ruleset") {
      const rulesetId = stringField(body, "rulesetId");
      return rulesetId ? { ...mutation, rulesetId } : null;
    }
    return mutation;
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
          message: "Repository branch policy action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await mutateRepositoryBranchSettingsFromCookie(
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
          code: "repository_branch_policy_failed",
          message: "Repository branch policy update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
