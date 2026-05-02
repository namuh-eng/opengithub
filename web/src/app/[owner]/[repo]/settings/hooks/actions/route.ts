import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  mutateRepositoryWebhookSettingsFromCookie,
  type RepositoryWebhookMutation,
  type RepositoryWebhookMutationPayload,
  type WebhookContentType,
  type WebhookEventSelection,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const actions = new Set([
  "create-webhook",
  "update-webhook",
  "delete-webhook",
  "ping-webhook",
  "redeliver-delivery",
]);
const contentTypes = new Set(["json", "form"]);
const eventSelections = new Set(["push", "everything", "selected"]);

function stringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : "";
}

function optionalStringField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return typeof value === "string" ? value.trim() : undefined;
}

function stringListField(input: Record<string, unknown>, field: string) {
  const value = input[field];
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

function parseWebhookPayload(
  body: Record<string, unknown>,
): RepositoryWebhookMutationPayload | null {
  const payloadUrl = stringField(body, "payloadUrl");
  const contentType = stringField(body, "contentType");
  const eventSelection = stringField(body, "eventSelection");
  if (!payloadUrl) return null;
  if (!contentTypes.has(contentType)) return null;
  if (!eventSelections.has(eventSelection)) return null;

  return {
    active: body.active !== false,
    contentType: contentType as WebhookContentType,
    events: stringListField(body, "events"),
    eventSelection: eventSelection as WebhookEventSelection,
    payloadUrl,
    secret: optionalStringField(body, "secret"),
    sslVerify: body.sslVerify !== false,
  };
}

function parseMutation(input: unknown): RepositoryWebhookMutation | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const body = input as Record<string, unknown>;
  const action = stringField(body, "action");
  if (!actions.has(action)) return null;

  if (action === "create-webhook") {
    const payload = parseWebhookPayload(body);
    return payload ? { action, ...payload } : null;
  }
  if (action === "update-webhook") {
    const hookId = stringField(body, "hookId");
    const payload = parseWebhookPayload(body);
    return hookId && payload ? { action, hookId, ...payload } : null;
  }
  if (action === "delete-webhook" || action === "ping-webhook") {
    const hookId = stringField(body, "hookId");
    return hookId ? { action, hookId } : null;
  }
  if (action === "redeliver-delivery") {
    const hookId = stringField(body, "hookId");
    const deliveryId = stringField(body, "deliveryId");
    return hookId && deliveryId ? { action, hookId, deliveryId } : null;
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
          message: "Repository webhook action is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const result = await mutateRepositoryWebhookSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
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
          code: "repository_webhook_failed",
          message: "Repository webhook update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
