"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { type FormEvent, useEffect, useMemo, useRef, useState } from "react";
import type {
  RepositoryCreationOptions,
  RepositoryCreationVisibilityOption,
  RepositoryNameAvailability,
  RepositoryOwnerType,
  RepositoryVisibility,
} from "@/lib/api";

type RepositoryCreateFormProps = {
  options: RepositoryCreationOptions;
};

const VISIBILITY_COPY: Record<RepositoryVisibility, string> = {
  public: "Anyone on the internet can see this repository.",
  private: "You choose who can see and commit to this repository.",
  internal: "Organization members can see this repository.",
};

const DEFAULT_VISIBILITY_OPTIONS: RepositoryCreationVisibilityOption[] = [
  { visibility: "public", enabled: true, reason: null },
  { visibility: "private", enabled: true, reason: null },
];

function normalizePreview(value: string) {
  return value
    .trim()
    .replace(/\s+/g, "-")
    .replace(/[^A-Za-z0-9._-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

function ownerKey(ownerType: RepositoryOwnerType, ownerId: string) {
  return `${ownerType}:${ownerId}`;
}

export function RepositoryCreateForm({ options }: RepositoryCreateFormProps) {
  const router = useRouter();
  const [selectedOwnerKey, setSelectedOwnerKey] = useState(() => {
    const firstOwner = options.owners[0];
    return firstOwner
      ? ownerKey(firstOwner.ownerType, firstOwner.id)
      : "user:missing";
  });
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<RepositoryVisibility>("public");
  const [templateSlug, setTemplateSlug] = useState("blank");
  const [initializeReadme, setInitializeReadme] = useState(false);
  const [gitignoreSlug, setGitignoreSlug] = useState("");
  const [gitignoreSearch, setGitignoreSearch] = useState("");
  const [gitignoreOpen, setGitignoreOpen] = useState(false);
  const [licenseSlug, setLicenseSlug] = useState("");
  const [availability, setAvailability] =
    useState<RepositoryNameAvailability | null>(null);
  const [availabilityStatus, setAvailabilityStatus] = useState<
    "idle" | "checking" | "error"
  >("idle");
  const [submitting, setSubmitting] = useState(false);
  const [nameError, setNameError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const gitignoreSearchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (gitignoreOpen) {
      gitignoreSearchRef.current?.focus();
    }
  }, [gitignoreOpen]);

  const selectedOwner = useMemo(
    () =>
      options.owners.find(
        (owner) => ownerKey(owner.ownerType, owner.id) === selectedOwnerKey,
      ) ?? options.owners[0],
    [options.owners, selectedOwnerKey],
  );

  const normalizedName = normalizePreview(name);
  const filteredGitignoreTemplates = options.gitignoreTemplates.filter(
    (template) =>
      `${template.displayName} ${template.description}`
        .toLowerCase()
        .includes(gitignoreSearch.trim().toLowerCase()),
  );
  const selectedTemplate = options.templates.find(
    (template) => template.slug === templateSlug,
  );
  const selectedGitignore = options.gitignoreTemplates.find(
    (template) => template.slug === gitignoreSlug,
  );
  const selectedLicense = options.licenseTemplates.find(
    (template) => template.slug === licenseSlug,
  );
  const visibilityOptions =
    selectedOwner?.visibilityOptions?.length === 0 ||
    !selectedOwner?.visibilityOptions
      ? DEFAULT_VISIBILITY_OPTIONS
      : selectedOwner.visibilityOptions;
  const selectedVisibilityOption = visibilityOptions.find(
    (option) => option.visibility === visibility,
  );

  useEffect(() => {
    if (
      selectedVisibilityOption?.enabled === false ||
      !visibilityOptions.some((option) => option.visibility === visibility)
    ) {
      setVisibility(
        visibilityOptions.find((option) => option.enabled)?.visibility ??
          "public",
      );
    }
  }, [selectedVisibilityOption, visibility, visibilityOptions]);

  async function checkAvailability(candidate = name) {
    if (!selectedOwner || !candidate.trim()) {
      setAvailability(null);
      return;
    }

    setNameError(null);
    setAvailabilityStatus("checking");
    setAvailability(null);
    try {
      const params = new URLSearchParams({
        ownerType: selectedOwner.ownerType,
        ownerId: selectedOwner.id,
        name: candidate,
      });
      const response = await fetch(`/new/name-availability?${params}`);
      if (!response.ok) {
        throw new Error("availability failed");
      }
      setAvailability((await response.json()) as RepositoryNameAvailability);
      setAvailabilityStatus("idle");
    } catch {
      setAvailabilityStatus("error");
    }
  }

  async function submitRepository(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setNameError(null);
    setFormError(null);

    if (!selectedOwner) {
      setFormError("Choose an owner before creating the repository.");
      return;
    }
    if (!normalizedName) {
      setNameError("Repository name is required.");
      return;
    }

    setSubmitting(true);
    try {
      const response = await fetch("/new/repositories", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          ownerType: selectedOwner.ownerType,
          ownerId: selectedOwner.id,
          name,
          description,
          visibility,
          defaultBranch: "main",
          initializeReadme,
          templateSlug,
          gitignoreTemplateSlug: gitignoreSlug || null,
          licenseTemplateSlug: licenseSlug || null,
        }),
      });
      const body = await response.json();
      if (!response.ok) {
        const message =
          body?.error?.message ?? "Repository could not be created.";
        if (
          body?.error?.code === "conflict" ||
          body?.error?.code === "validation_failed"
        ) {
          setNameError(message);
        } else {
          setFormError(message);
        }
        setSubmitting(false);
        return;
      }

      router.push(body.href ?? `/${selectedOwner.login}/${normalizedName}`);
    } catch {
      setFormError("Repository could not be created. Try again.");
      setSubmitting(false);
    }
  }

  const createDisabled = submitting || !selectedOwner || !normalizedName;

  const normalizedNameChanged =
    name.trim().length > 0 && name !== normalizedName;
  const descriptionAtLimit = description.length >= 350;

  return (
    <form
      aria-busy={submitting}
      className="mx-auto max-w-[760px] px-4 py-7 sm:px-6"
      onSubmit={(event) => void submitRepository(event)}
    >
      <header className="border-b pb-5" style={{ borderColor: "var(--line)" }}>
        <h1 className="t-h2" style={{ color: "var(--ink-1)" }}>
          Create a new repository
        </h1>
        <p className="mt-2 t-sm leading-5" style={{ color: "var(--ink-3)" }}>
          Repositories contain a project's files and version history. Have a
          project elsewhere?{" "}
          <Link
            className="hover:underline"
            href="/new/import"
            style={{ color: "var(--accent)" }}
          >
            Import a repository.
          </Link>
        </p>
        <p className="mt-1 t-sm italic" style={{ color: "var(--ink-3)" }}>
          Required fields are marked with an asterisk (*).
        </p>
      </header>

      <section className="grid grid-cols-[28px_minmax(0,1fr)] gap-x-3 pt-6 sm:grid-cols-[36px_minmax(0,1fr)] sm:gap-x-4">
        <div className="flex flex-col items-center">
          <span
            className="flex h-7 w-7 items-center justify-center rounded-full t-sm font-semibold"
            style={{ background: "var(--ink-4)", color: "var(--bg)" }}
          >
            1
          </span>
          <span
            className="mt-2 flex-1 border-l"
            style={{ borderColor: "var(--line)" }}
          />
        </div>
        <div className="pb-7">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            General
          </h2>
          <div className="mt-4 grid gap-4 sm:grid-cols-[140px_1fr]">
            <label className="block">
              <span
                className="t-sm font-semibold"
                style={{ color: "var(--ink-1)" }}
              >
                Owner *
              </span>
              <select
                className="input mt-2 h-9 w-full px-3 t-sm"
                value={selectedOwnerKey}
                onChange={(event) => {
                  setSelectedOwnerKey(event.target.value);
                  setAvailability(null);
                  setNameError(null);
                }}
              >
                {options.owners.map((owner) => (
                  <option
                    key={ownerKey(owner.ownerType, owner.id)}
                    value={ownerKey(owner.ownerType, owner.id)}
                  >
                    {owner.login}
                  </option>
                ))}
              </select>
            </label>
            <label className="block">
              <span
                className="t-sm font-semibold"
                style={{ color: "var(--ink-1)" }}
              >
                Repository name *
              </span>
              <input
                aria-describedby="repository-name-help repository-name-feedback"
                aria-invalid={nameError ? "true" : "false"}
                className="input mt-2 h-9 w-full px-3 t-sm"
                value={name}
                onBlur={() => void checkAvailability()}
                onChange={(event) => {
                  setName(event.target.value);
                  setAvailability(null);
                  setNameError(null);
                }}
                required
              />
            </label>
          </div>
          <p className="mt-3 t-sm" style={{ color: "var(--ink-3)" }}>
            <span id="repository-name-help">
              Great repository names are short and memorable. How about{" "}
            </span>
            <button
              className="font-medium hover:underline"
              type="button"
              style={{ color: "var(--ok)" }}
              onClick={() => {
                setName(options.suggestedName);
                void checkAvailability(options.suggestedName);
              }}
            >
              {options.suggestedName}
            </button>
            ?
          </p>
          <div id="repository-name-feedback">
            {normalizedNameChanged ? (
              <p
                className="mt-2 t-sm"
                role="status"
                style={{ color: "var(--ok)" }}
              >
                This will be normalized to{" "}
                <span className="t-mono" style={{ color: "var(--ink-1)" }}>
                  {normalizedName}
                </span>
                .
              </p>
            ) : null}
            {availabilityStatus === "checking" ? (
              <p
                className="mt-2 t-sm"
                role="status"
                style={{ color: "var(--ink-3)" }}
              >
                Checking repository name...
              </p>
            ) : null}
            {availabilityStatus === "error" ? (
              <p
                className="mt-2 t-sm"
                role="alert"
                style={{ color: "var(--err)" }}
              >
                Name availability could not be checked.
              </p>
            ) : null}
            {availability ? (
              <p
                className="mt-2 t-sm"
                role={availability.available ? "status" : "alert"}
                style={{
                  color: availability.available ? "var(--ok)" : "var(--err)",
                }}
              >
                {availability.available
                  ? `${availability.normalizedName} is available.`
                  : availability.reason}
              </p>
            ) : null}
            {nameError ? (
              <p
                className="mt-2 t-sm"
                role="alert"
                style={{ color: "var(--err)" }}
              >
                {nameError}
              </p>
            ) : null}
          </div>
          <label className="mt-4 block">
            <span
              className="t-sm font-semibold"
              style={{ color: "var(--ink-1)" }}
            >
              Description
            </span>
            <input
              className="input mt-2 h-9 w-full px-3 t-sm"
              maxLength={350}
              value={description}
              onChange={(event) => setDescription(event.target.value)}
            />
            <span className="mt-2 block t-xs" style={{ color: "var(--ink-3)" }}>
              <span
                className={descriptionAtLimit ? "font-semibold" : ""}
                style={
                  descriptionAtLimit ? { color: "var(--warn)" } : undefined
                }
              >
                {description.length}
              </span>{" "}
              / 350 characters
            </span>
          </label>
        </div>
      </section>

      <section className="grid grid-cols-[28px_minmax(0,1fr)] gap-x-3 sm:grid-cols-[36px_minmax(0,1fr)] sm:gap-x-4">
        <div className="flex flex-col items-center">
          <span
            className="flex h-7 w-7 items-center justify-center rounded-full t-sm font-semibold"
            style={{ background: "var(--ink-4)", color: "var(--bg)" }}
          >
            2
          </span>
          <span
            className="mt-2 flex-1 border-l"
            style={{ borderColor: "var(--line)" }}
          />
        </div>
        <div className="pb-6">
          <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
            Configuration
          </h2>
          <div
            className="mt-4 divide-y rounded-md"
            style={{
              border: "1px solid var(--line)",
              background: "var(--surface)",
              borderColor: "var(--line)",
            }}
          >
            <label className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
              <span>
                <span
                  className="block t-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  Choose visibility *
                </span>
                <span
                  className="mt-1 block t-sm"
                  style={{ color: "var(--ink-3)" }}
                >
                  {VISIBILITY_COPY[visibility]}
                </span>
              </span>
              <select
                className="input h-9 px-3 t-sm font-semibold"
                value={visibility}
                onChange={(event) =>
                  setVisibility(event.target.value as RepositoryVisibility)
                }
              >
                {visibilityOptions.map((option) => (
                  <option
                    disabled={!option.enabled}
                    key={option.visibility}
                    value={option.visibility}
                  >
                    {visibilityLabel(option.visibility)}
                    {option.enabled ? "" : " - disabled by organization policy"}
                  </option>
                ))}
              </select>
            </label>
            {visibilityOptions.some((option) => !option.enabled) ? (
              <div
                className="px-4 pb-4 pt-0 t-sm"
                style={{ color: "var(--ink-3)" }}
              >
                {visibilityOptions
                  .filter((option) => !option.enabled)
                  .map((option) => (
                    <p key={option.visibility}>
                      <span className="chip warn mr-2">
                        {visibilityLabel(option.visibility)}
                      </span>
                      {option.reason ??
                        "Organization policy has disabled this visibility."}
                    </p>
                  ))}
              </div>
            ) : null}

            <label className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
              <span>
                <span
                  className="block t-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  Start with a template
                </span>
                <span
                  className="mt-1 block t-sm"
                  style={{ color: "var(--ink-3)" }}
                >
                  {selectedTemplate?.description ??
                    "Templates pre-configure your repository with files."}
                </span>
              </span>
              <select
                className="input h-9 px-3 t-sm font-semibold"
                value={templateSlug}
                onChange={(event) => setTemplateSlug(event.target.value)}
              >
                {options.templates.map((template) => (
                  <option key={template.slug} value={template.slug}>
                    {template.displayName}
                  </option>
                ))}
              </select>
            </label>

            <div className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
              <span>
                <span
                  className="block t-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  Add README
                </span>
                <span
                  className="mt-1 block t-sm"
                  style={{ color: "var(--ink-3)" }}
                >
                  READMEs can be used as longer descriptions.
                </span>
              </span>
              <button
                aria-pressed={initializeReadme}
                className={`h-8 min-w-16 rounded-md border px-3 t-sm font-semibold ${
                  initializeReadme ? "chip ok" : "btn"
                }`}
                type="button"
                onClick={() => setInitializeReadme((value) => !value)}
              >
                {initializeReadme ? "On" : "Off"}
              </button>
            </div>

            <details
              className="p-4"
              open={gitignoreOpen}
              onToggle={(event) => setGitignoreOpen(event.currentTarget.open)}
            >
              <summary className="flex cursor-pointer list-none flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <span>
                  <span
                    className="block t-sm font-semibold"
                    style={{ color: "var(--ink-1)" }}
                  >
                    Add .gitignore
                  </span>
                  <span
                    className="mt-1 block t-sm"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {selectedGitignore?.description ??
                      ".gitignore tells git which files not to track."}
                  </span>
                </span>
                <span className="btn inline-flex h-9 items-center px-3 t-sm font-semibold">
                  {selectedGitignore?.displayName ?? "No .gitignore"}
                </span>
              </summary>
              <div
                className="mt-4 rounded-md p-3"
                style={{
                  border: "1px solid var(--line)",
                  background: "var(--surface-2)",
                }}
              >
                <label
                  className="block t-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  Search gitignore templates
                  <input
                    ref={gitignoreSearchRef}
                    className="input mt-2 h-9 w-full px-3 t-sm"
                    value={gitignoreSearch}
                    onChange={(event) => setGitignoreSearch(event.target.value)}
                  />
                </label>
                <div
                  aria-label="Gitignore templates"
                  className="mt-3 max-h-48 overflow-auto"
                  role="listbox"
                >
                  <button
                    className="block w-full rounded-md px-3 py-2 text-left t-sm hover:bg-[var(--surface)]"
                    role="option"
                    type="button"
                    aria-selected={gitignoreSlug === ""}
                    onClick={() => setGitignoreSlug("")}
                  >
                    No .gitignore
                  </button>
                  {filteredGitignoreTemplates.map((template) => (
                    <button
                      className="block w-full rounded-md px-3 py-2 text-left t-sm hover:bg-[var(--surface)]"
                      key={template.slug}
                      role="option"
                      type="button"
                      aria-selected={gitignoreSlug === template.slug}
                      onClick={() => setGitignoreSlug(template.slug)}
                    >
                      <span className="font-semibold">
                        {template.displayName}
                      </span>
                      <span
                        className="block t-xs"
                        style={{ color: "var(--ink-3)" }}
                      >
                        {template.description}
                      </span>
                    </button>
                  ))}
                  {filteredGitignoreTemplates.length === 0 ? (
                    <p
                      className="px-3 py-2 t-sm"
                      style={{ color: "var(--ink-3)" }}
                    >
                      No templates match this search.
                    </p>
                  ) : null}
                </div>
              </div>
            </details>

            <label className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
              <span>
                <span
                  className="block t-sm font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  Add license
                </span>
                <span
                  className="mt-1 block t-sm"
                  style={{ color: "var(--ink-3)" }}
                >
                  {selectedLicense?.description ??
                    "Licenses explain how others can use your code."}
                </span>
              </span>
              <select
                className="input h-9 px-3 t-sm font-semibold"
                value={licenseSlug}
                onChange={(event) => setLicenseSlug(event.target.value)}
              >
                <option value="">No license</option>
                {options.licenseTemplates.map((template) => (
                  <option key={template.slug} value={template.slug}>
                    {template.displayName}
                  </option>
                ))}
              </select>
            </label>
          </div>
        </div>
      </section>

      <div
        className="flex flex-col gap-3 border-t pt-5 sm:ml-[52px] sm:flex-row sm:justify-end"
        style={{ borderColor: "var(--line)" }}
      >
        {formError ? (
          <p
            className="self-center t-sm sm:mr-4"
            role="alert"
            style={{ color: "var(--err)" }}
          >
            {formError}
          </p>
        ) : null}
        <button
          className="btn primary h-9 px-4 t-sm font-semibold disabled:cursor-not-allowed disabled:opacity-60"
          disabled={createDisabled}
          type="submit"
        >
          {submitting ? "Creating..." : "Create repository"}
        </button>
      </div>
    </form>
  );
}

function visibilityLabel(visibility: RepositoryVisibility) {
  return visibility.charAt(0).toUpperCase() + visibility.slice(1);
}
