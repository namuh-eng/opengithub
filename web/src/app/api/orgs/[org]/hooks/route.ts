import { type NextRequest, NextResponse } from "next/server";
import { createOrganizationWebhookFromCookie } from "@/lib/api";

type Context = { params: Promise<{ org: string }> };

export async function POST(request: NextRequest, context: Context) {
  const { org } = await context.params;
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
      await createOrganizationWebhookFromCookie(
        request.headers.get("cookie"),
        org,
        input,
      ),
      { status: 201 },
    );
  } catch (error) {
    const envelope = (error instanceof Error ? error.cause : null) as {
      status?: number;
    } | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "webhook_failed",
          message: "Webhook could not be created",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
