import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositorySecurityAdvisoryFromCookie,
  type RepositorySecurityAdvisoryCreate,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function optionalStringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseCreate(input: unknown): RepositorySecurityAdvisoryCreate | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const title = stringField(body, "title");
  if (!title) return null;
  const cvssScoreRaw = body.cvssScore;
  return {
    title,
    summary: optionalStringField(body, "summary"),
    detailsMarkdown: optionalStringField(body, "detailsMarkdown"),
    cveId: optionalStringField(body, "cveId"),
    severity: optionalStringField(body, "severity"),
    packageEcosystem: optionalStringField(body, "packageEcosystem"),
    packageName: optionalStringField(body, "packageName"),
    affectedVersions: optionalStringField(body, "affectedVersions"),
    patchedVersions: optionalStringField(body, "patchedVersions"),
    cvssVector: optionalStringField(body, "cvssVector"),
    cvssScore:
      typeof cvssScoreRaw === "number" && Number.isFinite(cvssScoreRaw)
        ? cvssScoreRaw
        : null,
    cvssMetrics:
      body.cvssMetrics && typeof body.cvssMetrics === "object"
        ? (body.cvssMetrics as Record<string, unknown>)
        : null,
    cwes: [],
    credits: [],
    collaborators: [],
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const input = await request.json().catch(() => null);
  const draft = parseCreate(input);
  if (!draft) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "A draft security advisory requires a title.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const advisory = await createRepositorySecurityAdvisoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      draft,
    );
    return NextResponse.json(advisory, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "security_advisory_create_failed",
          message: "Repository security advisory creation failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
