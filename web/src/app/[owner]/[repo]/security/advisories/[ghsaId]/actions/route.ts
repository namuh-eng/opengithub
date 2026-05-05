import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositorySecurityAdvisoryFromCookie,
  type RepositorySecurityAdvisoryMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; ghsaId: string }>;
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
): RepositorySecurityAdvisoryMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const title = stringField(body, "title");
  const summary = stringField(body, "summary");
  const detailsMarkdown = stringField(body, "detailsMarkdown");
  const severity = stringField(body, "severity");
  if (!title || !summary || !detailsMarkdown || !severity) {
    return null;
  }
  const cwes = Array.isArray(body.cwes)
    ? body.cwes.flatMap((value) => {
        if (!value || typeof value !== "object" || Array.isArray(value)) {
          return [];
        }
        const row = value as Record<string, unknown>;
        const id = stringField(row, "id");
        if (!id) return [];
        return [
          {
            id,
            name: stringField(row, "name") || "Common Weakness Enumeration",
            href: optionalStringField(row, "href"),
          },
        ];
      })
    : [];
  const credits = Array.isArray(body.credits)
    ? body.credits.flatMap((value) => {
        if (!value || typeof value !== "object" || Array.isArray(value)) {
          return [];
        }
        const row = value as Record<string, unknown>;
        const login = stringField(row, "login");
        if (!login) return [];
        return [
          { login, creditType: stringField(row, "creditType") || "reporter" },
        ];
      })
    : [];
  const collaborators = Array.isArray(body.collaborators)
    ? body.collaborators.flatMap((value) => {
        if (!value || typeof value !== "object" || Array.isArray(value)) {
          return [];
        }
        const row = value as Record<string, unknown>;
        const login = stringField(row, "login");
        if (!login) return [];
        return [{ login, role: stringField(row, "role") || "collaborator" }];
      })
    : [];
  const cvssScoreRaw = body.cvssScore;
  const cvssScore =
    typeof cvssScoreRaw === "number" && Number.isFinite(cvssScoreRaw)
      ? cvssScoreRaw
      : null;

  return {
    title,
    summary,
    detailsMarkdown,
    cveId: optionalStringField(body, "cveId"),
    severity,
    packageEcosystem: optionalStringField(body, "packageEcosystem"),
    packageName: optionalStringField(body, "packageName"),
    affectedVersions: optionalStringField(body, "affectedVersions"),
    patchedVersions: optionalStringField(body, "patchedVersions"),
    cvssVector: optionalStringField(body, "cvssVector"),
    cvssScore,
    cvssMetrics:
      body.cvssMetrics && typeof body.cvssMetrics === "object"
        ? (body.cvssMetrics as Record<string, unknown>)
        : {},
    cwes,
    credits,
    collaborators,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, ghsaId } = await context.params;
  const input = await request.json().catch(() => null);
  const mutation = parseMutation(input);
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Security advisory metadata is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const advisory = await mutateRepositorySecurityAdvisoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(ghsaId),
      mutation,
    );
    return NextResponse.json(advisory);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "security_advisory_update_failed",
          message: "Repository security advisory update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
