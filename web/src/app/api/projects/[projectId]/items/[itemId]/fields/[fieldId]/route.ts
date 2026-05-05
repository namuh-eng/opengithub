import { type NextRequest, NextResponse } from "next/server";
import { updateProjectItemFieldFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{
    projectId: string;
    itemId: string;
    fieldId: string;
  }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, itemId, fieldId } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const workspace = await updateProjectItemFieldFromCookie(
      request.headers.get("cookie"),
      projectId,
      itemId,
      fieldId,
      body,
    );
    return NextResponse.json(workspace);
  } catch (error) {
    const cause = error instanceof Error ? error.cause : null;
    const envelope =
      cause &&
      typeof cause === "object" &&
      "error" in cause &&
      "status" in cause
        ? (cause as {
            error: { code: string; message: string };
            status: number;
          })
        : {
            error: {
              code: "project_item_field_failed",
              message: "Project item field could not be saved.",
            },
            status: 500,
          };
    return NextResponse.json(envelope, { status: envelope.status });
  }
}
