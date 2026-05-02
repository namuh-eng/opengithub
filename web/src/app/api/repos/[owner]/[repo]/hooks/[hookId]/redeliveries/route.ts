import { type NextRequest, NextResponse } from "next/server";
import { redeliverRepositoryWebhookFromCookie } from "@/lib/api";

type Context = {
  params: Promise<{ owner: string; repo: string; hookId: string }>;
};

export async function POST(request: NextRequest, context: Context) {
  const { owner, repo, hookId } = await context.params;
  const input = await request.json().catch(() => ({}));
  try {
    return NextResponse.json(
      await redeliverRepositoryWebhookFromCookie(
        request.headers.get("cookie"),
        owner,
        repo,
        hookId,
        input.deliveryId,
      ),
      { status: 202 },
    );
  } catch (error) {
    const envelope = (error instanceof Error ? error.cause : null) as {
      status?: number;
    } | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "webhook_failed",
          message: "Webhook delivery could not be retried",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
