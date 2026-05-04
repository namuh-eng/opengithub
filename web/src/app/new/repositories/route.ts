import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateRepositoryRequest,
  createRepositoryFromCookie,
} from "@/lib/api";

const OWNER_TYPES = new Set(["user", "organization"]);
const VISIBILITIES = new Set(["public", "private", "internal"]);

function validationError(message: string) {
  return NextResponse.json(
    {
      error: {
        code: "validation_failed",
        message,
      },
      status: 422,
    },
    { status: 422 },
  );
}

export async function POST(request: NextRequest) {
  let body: CreateRepositoryRequest;
  try {
    body = (await request.json()) as CreateRepositoryRequest;
  } catch {
    return validationError("Repository creation payload must be valid JSON");
  }

  if (!OWNER_TYPES.has(body.ownerType) || !body.ownerId || !body.name?.trim()) {
    return validationError("Owner and repository name are required");
  }
  if (!VISIBILITIES.has(body.visibility)) {
    return validationError("Visibility must be public, private, or internal");
  }

  try {
    const repository = await createRepositoryFromCookie(
      request.headers.get("cookie"),
      body,
    );
    return NextResponse.json(repository, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_create_failed",
          message: "Repository could not be created",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
