import { type NextRequest, NextResponse } from "next/server";
import { updateOrganizationWebhookFromCookie } from "@/lib/api";

type Context = { params: Promise<{ org: string; hookId: string }> };

export async function PATCH(request: NextRequest, context: Context) {
  const { org, hookId } = await context.params;
  const input = await request.json().catch(() => null);
  if (!input)
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON",
        },
        status: 400,
      },
      { status: 400 },
    );
  try {
    return NextResponse.json(
      await updateOrganizationWebhookFromCookie(
        request.headers.get("cookie"),
        org,
        hookId,
        input,
      ),
    );
  } catch (error) {
    const envelope = (error instanceof Error ? error.cause : null) as {
      status?: number;
    } | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "webhook_failed",
          message: "Webhook could not be updated",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
