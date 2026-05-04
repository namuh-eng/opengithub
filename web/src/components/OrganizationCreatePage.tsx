"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  type ChangeEvent,
  type FormEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { OrganizationSlugAvailability } from "@/lib/api";

type PlanId = "free" | "team" | "enterprise";
type OwnershipType = "personal" | "business";
type AvailabilityStatus = "idle" | "checking" | "error";
type FieldErrors = Partial<
  Record<"name" | "contactEmail" | "companyName" | "termsAccepted", string>
>;

const PLANS: Array<{
  id: PlanId;
  name: string;
  price: string;
  summary: string;
  details: string[];
  disabled?: boolean;
}> = [
  {
    id: "free",
    name: "Free",
    price: "$0",
    summary: "Open source collaboration with public and private repositories.",
    details: [
      "Unlimited members",
      "Issues and pull requests",
      "Community Actions minutes",
    ],
  },
  {
    id: "team",
    name: "Team",
    price: "Later",
    summary: "Advanced controls for growing engineering teams.",
    details: ["Required reviewers", "Code owners", "SAML and policy controls"],
    disabled: true,
  },
  {
    id: "enterprise",
    name: "Enterprise",
    price: "Contact",
    summary: "Centralized governance for many organizations.",
    details: ["Managed users", "Audit exports", "Dedicated support"],
    disabled: true,
  },
];

function normalizePreview(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 39);
}

function availabilityCopy(
  name: string,
  status: AvailabilityStatus,
  availability: OrganizationSlugAvailability | null,
) {
  if (!name.trim()) {
    return "Type an organization name to preview its URL.";
  }
  if (status === "checking") {
    return "Checking organization URL availability...";
  }
  if (status === "error") {
    return "Availability could not be checked. Try again in a moment.";
  }
  if (!availability) {
    return "Blur the name field to confirm availability.";
  }
  if (availability.available) {
    return `${availability.normalizedSlug} is available.`;
  }
  return availability.reason ?? "This organization URL is not available.";
}

export function OrganizationCreatePage() {
  const router = useRouter();
  const [step, setStep] = useState<"plans" | "setup">("plans");
  const [name, setName] = useState("");
  const [contactEmail, setContactEmail] = useState("");
  const [ownershipType, setOwnershipType] = useState<OwnershipType>("personal");
  const [companyName, setCompanyName] = useState("");
  const [termsAccepted, setTermsAccepted] = useState(false);
  const [availability, setAvailability] =
    useState<OrganizationSlugAvailability | null>(null);
  const [availabilityStatus, setAvailabilityStatus] =
    useState<AvailabilityStatus>("idle");
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<FieldErrors>({});
  const setupHeadingRef = useRef<HTMLHeadingElement>(null);

  const normalizedSlug = availability?.normalizedSlug || normalizePreview(name);
  const availabilityMessage = availabilityCopy(
    name,
    availabilityStatus,
    availability,
  );
  const availabilityTone =
    availabilityStatus === "error" || availability?.available === false
      ? "err"
      : availability?.available
        ? "ok"
        : "soft";
  const emailInvalid =
    contactEmail.trim().length > 0 &&
    !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(contactEmail);
  const canCreate =
    !submitting &&
    Boolean(name.trim()) &&
    Boolean(normalizedSlug) &&
    availability?.available === true &&
    Boolean(contactEmail.trim()) &&
    !emailInvalid &&
    (ownershipType === "personal" || Boolean(companyName.trim())) &&
    termsAccepted;

  useEffect(() => {
    if (step === "setup") {
      setupHeadingRef.current?.focus();
    }
  }, [step]);

  useEffect(() => {
    setAvailability(null);
    setAvailabilityStatus("idle");
    if (!name.trim()) {
      return;
    }

    const abort = new AbortController();
    const timeout = window.setTimeout(async () => {
      setAvailabilityStatus("checking");
      try {
        const params = new URLSearchParams({ name });
        const response = await fetch(
          `/organizations/new/slug-availability?${params}`,
          { signal: abort.signal },
        );
        if (!response.ok) {
          throw new Error("availability failed");
        }
        const body = (await response.json()) as OrganizationSlugAvailability;
        setAvailability(body);
        setAvailabilityStatus("idle");
      } catch {
        if (!abort.signal.aborted) {
          setAvailabilityStatus("error");
        }
      }
    }, 300);

    return () => {
      abort.abort();
      window.clearTimeout(timeout);
    };
  }, [name]);

  const previewHref = useMemo(
    () => `opengithub.namuh.co/${normalizedSlug || "your-org"}`,
    [normalizedSlug],
  );

  function chooseFreePlan() {
    setStep("setup");
  }

  function updateOwnership(event: ChangeEvent<HTMLInputElement>) {
    const next = event.target.value as OwnershipType;
    setFormError(null);
    setFieldErrors({});
    setOwnershipType(next);
    if (next === "personal") {
      setCompanyName("");
    }
  }

  function fieldForApiError(message: string) {
    const normalized = message.toLowerCase();
    if (
      normalized.includes("slug") ||
      normalized.includes("organization name")
    ) {
      return "name" as const;
    }
    if (normalized.includes("email")) {
      return "contactEmail" as const;
    }
    if (normalized.includes("company") || normalized.includes("institution")) {
      return "companyName" as const;
    }
    if (normalized.includes("terms")) {
      return "termsAccepted" as const;
    }
    return null;
  }

  function validateFields(): FieldErrors {
    const errors: FieldErrors = {};
    if (!name.trim() || !normalizedSlug) {
      errors.name = "Organization name is required.";
    } else if (availability?.available !== true) {
      errors.name =
        availability?.reason ??
        "Confirm an available organization URL before creating.";
    }
    if (!contactEmail.trim()) {
      errors.contactEmail = "Contact email is required.";
    } else if (emailInvalid) {
      errors.contactEmail = "Enter a valid contact email.";
    }
    if (ownershipType === "business" && !companyName.trim()) {
      errors.companyName =
        "Company or institution name is required for business organizations.";
    }
    if (!termsAccepted) {
      errors.termsAccepted =
        "Accept the organization terms before creating an organization.";
    }
    return errors;
  }

  async function submitSetup(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (submitting) {
      return;
    }

    setFormError(null);
    setFieldErrors({});
    const nextFieldErrors = validateFields();
    if (Object.keys(nextFieldErrors).length > 0) {
      setFieldErrors(nextFieldErrors);
      setFormError(
        "Complete the required fields and confirm the organization URL before creating.",
      );
      return;
    }

    setSubmitting(true);
    try {
      const response = await fetch("/organizations/new/create", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          name,
          contactEmail,
          ownershipType,
          companyName: ownershipType === "business" ? companyName : null,
          termsAccepted,
        }),
      });
      const body = await response.json();
      if (!response.ok) {
        const message =
          body?.error?.message ?? "Organization could not be created.";
        const field =
          (body?.details?.field as keyof FieldErrors | undefined) ??
          fieldForApiError(message);
        if (field) {
          setFieldErrors({ [field]: message });
        } else {
          setFormError(message);
        }
        if (body?.error?.code === "conflict" || field === "name") {
          setAvailability((current) =>
            current
              ? {
                  ...current,
                  available: false,
                  reason: message,
                  existingKind: "organization",
                }
              : current,
          );
        }
        setSubmitting(false);
        return;
      }

      router.push(body.href ?? `/orgs/${body.slug}`);
    } catch {
      setFormError("Organization could not be created. Try again.");
      setSubmitting(false);
    }
  }

  return (
    <section className="mx-auto w-full max-w-[1040px] px-4 py-8 sm:px-6">
      <div className="mb-8">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Organizations
        </p>
        <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
          Create a new organization
        </h1>
        <p
          className="mt-3 max-w-[680px] t-body"
          style={{ color: "var(--ink-3)" }}
        >
          Start with a Free organization, then invite collaborators and move
          repositories into a shared workspace.
        </p>
      </div>

      {step === "plans" ? (
        <div className="grid gap-4 lg:grid-cols-3">
          {PLANS.map((plan) => (
            <article
              aria-label={`${plan.name} plan`}
              className="card flex min-h-[360px] flex-col p-5"
              key={plan.id}
            >
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="t-label" style={{ color: "var(--ink-3)" }}>
                    {plan.id === "free" ? "MVP" : "Info only"}
                  </p>
                  <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
                    {plan.name}
                  </h2>
                </div>
                <span className={plan.disabled ? "chip soft" : "chip active"}>
                  {plan.price}
                </span>
              </div>
              <p className="mt-4 t-body" style={{ color: "var(--ink-2)" }}>
                {plan.summary}
              </p>
              <ul className="mt-5 grid gap-3">
                {plan.details.map((detail) => (
                  <li className="flex gap-2 t-sm" key={detail}>
                    <span aria-hidden="true" style={{ color: "var(--accent)" }}>
                      -
                    </span>
                    <span style={{ color: "var(--ink-2)" }}>{detail}</span>
                  </li>
                ))}
              </ul>
              <div className="mt-auto pt-6">
                <button
                  className={
                    plan.disabled ? "btn w-full" : "btn primary w-full"
                  }
                  disabled={plan.disabled}
                  onClick={plan.disabled ? undefined : chooseFreePlan}
                  type="button"
                >
                  {plan.disabled
                    ? `${plan.name} plan unavailable`
                    : "Create a free organization"}
                </button>
                {plan.disabled ? (
                  <p className="mt-3 t-xs" style={{ color: "var(--ink-3)" }}>
                    Paid plan provisioning is outside this MVP.
                  </p>
                ) : null}
              </div>
            </article>
          ))}
        </div>
      ) : (
        <form
          aria-describedby={formError ? "organization-form-error" : undefined}
          className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]"
          onSubmit={submitSetup}
        >
          <div className="card p-5 sm:p-6">
            <button
              className="btn ghost sm mb-5"
              onClick={() => setStep("plans")}
              type="button"
            >
              Back to plans
            </button>
            <h2
              className="t-h2 outline-none"
              ref={setupHeadingRef}
              tabIndex={-1}
            >
              Tell us about your organization
            </h2>
            <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
              Required fields are marked with an asterisk (*).
            </p>

            {formError ? (
              <div
                className="chip err mt-5 rounded-md px-4 py-3 t-sm"
                id="organization-form-error"
                role="alert"
                style={{
                  background: "var(--err-soft)",
                  color: "var(--err)",
                  border: "none",
                }}
              >
                {formError}
              </div>
            ) : null}

            <div className="mt-6 grid gap-5">
              <label className="grid gap-2">
                <span className="t-label" style={{ color: "var(--ink-3)" }}>
                  Organization name *
                </span>
                <span className="input">
                  <input
                    aria-describedby="organization-url-preview organization-availability"
                    aria-invalid={fieldErrors.name ? "true" : "false"}
                    autoComplete="organization"
                    onChange={(event) => {
                      setFormError(null);
                      setFieldErrors((current) => ({
                        ...current,
                        name: undefined,
                      }));
                      setName(event.target.value);
                    }}
                    placeholder="Acme Labs"
                    value={name}
                  />
                </span>
              </label>
              <div className="grid gap-2">
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Organization URL
                </p>
                <div
                  className="rounded-md px-3 py-2 t-mono-sm"
                  id="organization-url-preview"
                  style={{
                    background: "var(--surface-2)",
                    border: "1px solid var(--line)",
                    color: "var(--ink-2)",
                    overflowWrap: "anywhere",
                  }}
                >
                  {previewHref}
                </div>
                <p
                  className={`chip ${availabilityTone} w-fit`}
                  id="organization-availability"
                  role="status"
                >
                  {fieldErrors.name ?? availabilityMessage}
                </p>
              </div>

              <label className="grid gap-2">
                <span className="t-label" style={{ color: "var(--ink-3)" }}>
                  Contact email *
                </span>
                <span className="input">
                  <input
                    aria-describedby={
                      fieldErrors.contactEmail || emailInvalid
                        ? "contact-email-error"
                        : undefined
                    }
                    aria-invalid={
                      fieldErrors.contactEmail || emailInvalid
                        ? "true"
                        : "false"
                    }
                    inputMode="email"
                    onChange={(event) => {
                      setFormError(null);
                      setFieldErrors((current) => ({
                        ...current,
                        contactEmail: undefined,
                      }));
                      setContactEmail(event.target.value);
                    }}
                    placeholder="admin@example.com"
                    type="email"
                    value={contactEmail}
                  />
                </span>
                {fieldErrors.contactEmail || emailInvalid ? (
                  <span
                    className="t-sm"
                    id="contact-email-error"
                    role="alert"
                    style={{ color: "var(--err)" }}
                  >
                    {fieldErrors.contactEmail ?? "Enter a valid contact email."}
                  </span>
                ) : null}
              </label>

              <fieldset className="grid gap-3">
                <legend className="t-label" style={{ color: "var(--ink-3)" }}>
                  Ownership type *
                </legend>
                <label className="card flex cursor-pointer gap-3 p-4">
                  <input
                    aria-label="Personal account"
                    checked={ownershipType === "personal"}
                    name="ownershipType"
                    onChange={updateOwnership}
                    type="radio"
                    value="personal"
                  />
                  <span>
                    <span className="block t-h3">Personal account</span>
                    <span
                      className="block t-sm"
                      style={{ color: "var(--ink-3)" }}
                    >
                      Owned by you for independent projects.
                    </span>
                  </span>
                </label>
                <label className="card flex cursor-pointer gap-3 p-4">
                  <input
                    aria-label="Business or institution"
                    checked={ownershipType === "business"}
                    name="ownershipType"
                    onChange={updateOwnership}
                    type="radio"
                    value="business"
                  />
                  <span>
                    <span className="block t-h3">Business or institution</span>
                    <span
                      className="block t-sm"
                      style={{ color: "var(--ink-3)" }}
                    >
                      Records a company name for policy and audit context.
                    </span>
                  </span>
                </label>
              </fieldset>

              {ownershipType === "business" ? (
                <label className="grid gap-2">
                  <span className="t-label" style={{ color: "var(--ink-3)" }}>
                    Company name *
                  </span>
                  <span className="input">
                    <input
                      aria-invalid={fieldErrors.companyName ? "true" : "false"}
                      aria-describedby={
                        fieldErrors.companyName
                          ? "company-name-error"
                          : undefined
                      }
                      onChange={(event) => {
                        setFormError(null);
                        setFieldErrors((current) => ({
                          ...current,
                          companyName: undefined,
                        }));
                        setCompanyName(event.target.value);
                      }}
                      placeholder="Acme Inc."
                      value={companyName}
                    />
                  </span>
                  {fieldErrors.companyName ? (
                    <span
                      className="t-sm"
                      id="company-name-error"
                      role="alert"
                      style={{ color: "var(--err)" }}
                    >
                      {fieldErrors.companyName}
                    </span>
                  ) : null}
                </label>
              ) : null}

              <label className="card flex cursor-pointer gap-3 p-4">
                <input
                  aria-describedby={
                    fieldErrors.termsAccepted ? "terms-error" : undefined
                  }
                  aria-invalid={fieldErrors.termsAccepted ? "true" : "false"}
                  aria-label="I accept the organization terms for this Free plan."
                  checked={termsAccepted}
                  onChange={(event) => {
                    setFormError(null);
                    setFieldErrors((current) => ({
                      ...current,
                      termsAccepted: undefined,
                    }));
                    setTermsAccepted(event.target.checked);
                  }}
                  type="checkbox"
                />
                <span className="t-sm" style={{ color: "var(--ink-2)" }}>
                  I accept the organization terms for this Free plan.
                </span>
              </label>
              {fieldErrors.termsAccepted ? (
                <p
                  className="t-sm"
                  id="terms-error"
                  role="alert"
                  style={{ color: "var(--err)" }}
                >
                  {fieldErrors.termsAccepted}
                </p>
              ) : null}
            </div>

            <div className="mt-6 flex flex-wrap gap-2">
              <button
                className="btn primary"
                disabled={!canCreate}
                type="submit"
              >
                {submitting ? "Creating..." : "Create organization"}
              </button>
              <Link className="btn" href="/dashboard">
                Cancel
              </Link>
            </div>
          </div>

          <aside className="grid content-start gap-4">
            <div className="card p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Selected plan
              </p>
              <h3 className="t-h2 mt-2">Free</h3>
              <p className="mt-3 t-sm" style={{ color: "var(--ink-3)" }}>
                Includes repository collaboration, issues, pull requests, and
                Actions surfaces already available in opengithub.
              </p>
            </div>
            <div className="card p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Next step
              </p>
              <p className="mt-2 t-sm" style={{ color: "var(--ink-2)" }}>
                Creating the organization adds you as owner and opens the new
                organization profile after persistence succeeds.
              </p>
            </div>
          </aside>
        </form>
      )}
    </section>
  );
}
