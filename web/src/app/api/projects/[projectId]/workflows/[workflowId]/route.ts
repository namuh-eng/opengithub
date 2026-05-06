import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectWorkflowUpdateRequest,
  updateProjectWorkflowFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; workflowId: string }>;
};

function parseRequest(input: unknown): ProjectWorkflowUpdateRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const request: ProjectWorkflowUpdateRequest = {};
  if (typeof body.enabled === "boolean") request.enabled = body.enabled;
  if (typeof body.condition === "string") request.condition = body.condition;
  if (typeof body.statusFieldId === "string" || body.statusFieldId === null) {
    request.statusFieldId = body.statusFieldId;
  }
  if (typeof body.statusOptionId === "string" || body.statusOptionId === null) {
    request.statusOptionId = body.statusOptionId;
  }
  if (Array.isArray(body.repositoryTargetIds)) {
    request.repositoryTargetIds = body.repositoryTargetIds.filter(
      (value): value is string => typeof value === "string",
    );
  }
  if (
    typeof body.archiveAfterDays === "number" ||
    body.archiveAfterDays === null
  ) {
    request.archiveAfterDays = body.archiveAfterDays;
  }
  if (typeof body.closeOnStatus === "boolean") {
    request.closeOnStatus = body.closeOnStatus;
  }
  if (
    typeof body.expectedUpdatedAt === "string" ||
    body.expectedUpdatedAt === null
  ) {
    request.expectedUpdatedAt = body.expectedUpdatedAt;
  }
  return request;
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, workflowId } = await context.params;
  const mutation = parseRequest(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Workflow update payload is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateProjectWorkflowFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(workflowId),
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
          code: "project_workflow_update_failed",
          message: "Project workflow could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
