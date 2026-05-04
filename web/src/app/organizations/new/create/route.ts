import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateOrganizationRequest,
  createOrganizationFromCookie,
} from "@/lib/api";

const OWNERSHIP_TYPES = new Set(["personal", "business"]);

function validationError(message: string, field?: string) {
  return NextResponse.json(
    {
      error: {
        code: "validation_failed",
        message,
      },
      status: 422,
      details: field ? { field } : null,
    },
    { status: 422 },
  );
}

export async function POST(request: NextRequest) {
  let body: CreateOrganizationRequest;
  try {
    body = (await request.json()) as CreateOrganizationRequest;
  } catch {
    return validationError("Organization creation payload must be valid JSON");
  }

  if (!body.name?.trim()) {
    return validationError("Organization name is required.", "name");
  }
  if (!body.contactEmail?.trim()) {
    return validationError("Contact email is required.", "contactEmail");
  }
  if (!OWNERSHIP_TYPES.has(body.ownershipType)) {
    return validationError("Ownership type is required.", "ownershipType");
  }
  if (body.ownershipType === "business" && !body.companyName?.trim()) {
    return validationError(
      "Company or institution name is required for business organizations.",
      "companyName",
    );
  }
  if (!body.termsAccepted) {
    return validationError(
      "You must accept the organization terms before creating an organization.",
      "termsAccepted",
    );
  }

  try {
    const organization = await createOrganizationFromCookie(
      request.headers.get("cookie"),
      body,
    );
    return NextResponse.json(organization, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "organization_create_failed",
          message: "Organization could not be created",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
