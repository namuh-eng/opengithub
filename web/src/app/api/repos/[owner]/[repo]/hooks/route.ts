import { type NextRequest, NextResponse } from "next/server";
import { createRepositoryWebhookFromCookie } from "@/lib/api";

type Context = { params: Promise<{ owner: string; repo: string }> };

export async function POST(request: NextRequest, context: Context) {
  const { owner, repo } = await context.params;
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
    const hook = await createRepositoryWebhookFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      input,
    );
    return NextResponse.json(hook, { status: 201 });
  } catch (error) {
    return proxyError(error, "Webhook could not be created");
  }
}

function proxyError(error: unknown, fallback: string) {
  const envelope = (error instanceof Error ? error.cause : null) as {
    status?: number;
  } | null;
  return NextResponse.json(
    envelope ?? {
      error: { code: "webhook_failed", message: fallback },
      status: 502,
    },
    { status: envelope?.status ?? 502 },
  );
}
