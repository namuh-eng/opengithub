import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateOrganizationWebhookSettingsFromCookie,
  type RepositoryWebhookMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ org: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { org } = await context.params;
  const mutation = (await request
    .json()
    .catch(() => null)) as RepositoryWebhookMutation | null;
  if (!mutation || typeof mutation !== "object" || !("action" in mutation)) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Organization webhook action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const result = await mutateOrganizationWebhookSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(org),
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
          code: "organization_webhook_failed",
          message: "Organization webhook update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
