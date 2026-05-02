"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type {
  BranchPolicyEnforcement,
  BranchPolicyMutationRequirements,
  BranchPolicyRequirements,
  BypassActor,
  RepositoryBranchPolicyMutation,
  RepositoryBranchRule,
  RepositoryBranchSettings,
  RepositoryBranchSettingsFetchResult,
  RepositoryOverview,
  RepositoryRuleset,
} from "@/lib/api";

type RepositoryBranchSettingsPageProps = {
  intent?: "rule" | "ruleset";
  repository: RepositoryOverview;
  settingsResult: RepositoryBranchSettingsFetchResult;
};

type PolicyCard =
  | { kind: "rule"; item: RepositoryBranchRule }
  | { kind: "ruleset"; item: RepositoryRuleset };
type EditorMode =
  | { kind: "rule"; item?: RepositoryBranchRule }
  | { kind: "ruleset"; item?: RepositoryRuleset }
  | null;
type Confirmation =
  | { kind: "rule"; item: RepositoryBranchRule }
  | { kind: "ruleset"; item: RepositoryRuleset }
  | null;

const requirementLabels: Array<{
  label: string;
  test: (requirements: BranchPolicyRequirements) => boolean;
}> = [
  {
    label: "Require up-to-date branch",
    test: (requirements) => requirements.requiresUpToDateBranch,
  },
  {
    label: "Conversation resolution",
    test: (requirements) => requirements.requiresConversationResolution,
  },
  {
    label: "Signed commits",
    test: (requirements) => requirements.requiresSignedCommits,
  },
  {
    label: "Linear history",
    test: (requirements) => requirements.requiresLinearHistory,
  },
  {
    label: "Merge queue",
    test: (requirements) => requirements.requiresMergeQueue,
  },
  {
    label: "Deployments",
    test: (requirements) => requirements.requiresDeployments,
  },
  {
    label: "Locked branch",
    test: (requirements) => requirements.locked,
  },
  {
    label: "Restrict pushes",
    test: (requirements) => requirements.restrictsPushes,
  },
];

function policyTitle(card: PolicyCard) {
  return card.kind === "rule" ? card.item.pattern : card.item.name;
}

function policyPatterns(card: PolicyCard) {
  return card.kind === "rule" ? [card.item.pattern] : card.item.patterns;
}

function enforcementLabel(enforcement: BranchPolicyEnforcement) {
  const labels: Record<BranchPolicyEnforcement, string> = {
    active: "Active",
    disabled: "Disabled",
    evaluate: "Evaluate",
  };
  return labels[enforcement];
}

function enforcementChipClass(enforcement: BranchPolicyEnforcement) {
  if (enforcement === "active") return "chip ok";
  if (enforcement === "evaluate") return "chip warn";
  return "chip soft";
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function requirementChips(requirements: BranchPolicyRequirements) {
  const chips: string[] = [];
  if (requirements.requiredApprovingReviewCount > 0) {
    chips.push(`${requirements.requiredApprovingReviewCount}+ reviews`);
  }
  for (const context of requirements.requiredStatusChecks) {
    chips.push(`check: ${context}`);
  }
  for (const environment of requirements.requiredDeploymentEnvironments) {
    chips.push(`deploy: ${environment}`);
  }
  for (const requirement of requirementLabels) {
    if (requirement.test(requirements)) chips.push(requirement.label);
  }
  if (requirements.allowsForcePushes) chips.push("Force pushes allowed");
  if (requirements.allowsDeletions) chips.push("Deletions allowed");
  return chips;
}

function emptyRequirements(): BranchPolicyRequirements {
  return {
    allowsDeletions: false,
    allowsForcePushes: false,
    locked: false,
    requiredApprovingReviewCount: 0,
    requiredDeploymentEnvironments: [],
    requiredStatusChecks: [],
    requiresConversationResolution: false,
    requiresDeployments: false,
    requiresLinearHistory: false,
    requiresMergeQueue: false,
    requiresSignedCommits: false,
    requiresUpToDateBranch: false,
    restrictsPushes: false,
  };
}

function splitList(value: FormDataEntryValue | null) {
  return String(value ?? "")
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function bypassActorsFromForm(value: FormDataEntryValue | null) {
  return String(value ?? "")
    .split(/\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [actorType = "", actorId = "", ...labelParts] = line.split(":");
      return {
        actorId: actorId.trim(),
        actorType: actorType.trim(),
        label: labelParts.join(":").trim(),
      };
    })
    .filter((actor) => actor.actorId && actor.actorType && actor.label);
}

function bypassActorsToText(actors: BypassActor[]) {
  return actors
    .map((actor) => `${actor.actorType}:${actor.actorId}:${actor.label}`)
    .join("\n");
}

function requirementsFromForm(
  form: FormData,
): BranchPolicyMutationRequirements {
  return {
    allowsDeletions: form.get("allowsDeletions") === "on",
    allowsForcePushes: form.get("allowsForcePushes") === "on",
    locked: form.get("locked") === "on",
    requiredApprovingReviewCount: Number(
      form.get("requiredApprovingReviewCount") ?? 0,
    ),
    requiredDeploymentEnvironments: splitList(
      form.get("requiredDeploymentEnvironments"),
    ),
    requiredStatusChecks: splitList(form.get("requiredStatusChecks")),
    requiresConversationResolution:
      form.get("requiresConversationResolution") === "on",
    requiresDeployments: form.get("requiresDeployments") === "on",
    requiresLinearHistory: form.get("requiresLinearHistory") === "on",
    requiresMergeQueue: form.get("requiresMergeQueue") === "on",
    requiresSignedCommits: form.get("requiresSignedCommits") === "on",
    requiresUpToDateBranch: form.get("requiresUpToDateBranch") === "on",
    restrictsPushes: form.get("restrictsPushes") === "on",
  };
}

function BranchSettingsUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryBranchSettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${isForbidden ? "warn" : "err"}`}>
        {isForbidden ? "Read access required" : "Unavailable"}
      </span>
      <h2 className="t-h2 mt-4">
        {isForbidden
          ? "Branch policies are restricted"
          : "Branch policies could not load"}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {isForbidden
          ? "Only users who can read this repository can view branch policy explanations."
          : result.message}
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Link
          className="btn"
          href={`/${repository.owner_login}/${repository.name}`}
        >
          Repository Code
        </Link>
        <Link className="btn" href="/dashboard">
          Dashboard
        </Link>
      </div>
    </section>
  );
}

function checkboxLabel(
  name: keyof BranchPolicyRequirements,
  label: string,
  requirements: BranchPolicyRequirements,
) {
  return (
    <label className="flex items-start gap-2">
      <input
        className="mt-1"
        defaultChecked={Boolean(requirements[name])}
        name={name}
        type="checkbox"
      />
      <span className="t-sm">{label}</span>
    </label>
  );
}

function PolicyEditor({
  busy,
  error,
  mode,
  onCancel,
  onSubmit,
  settings,
}: {
  busy: boolean;
  error: string | null;
  mode: Exclude<EditorMode, null>;
  onCancel: () => void;
  onSubmit: (mutation: RepositoryBranchPolicyMutation) => void;
  settings: RepositoryBranchSettings;
}) {
  const isRule = mode.kind === "rule";
  const item = mode.item;
  const requirements = item?.requirements ?? emptyRequirements();
  const title = isRule
    ? item
      ? "Edit branch protection rule"
      : "Branch protection rule editor"
    : item
      ? "Edit repository ruleset"
      : "Repository ruleset editor";

  return (
    <section className="card p-5" id="branch-policy-editor">
      <span className="chip active">{isRule ? "Branch rule" : "Ruleset"}</span>
      <h2 className="t-h3 mt-3">{title}</h2>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        Saves are confirmed by the Rust API before this page updates. Status
        checks accept comma or newline separated contexts.
      </p>
      <form
        className="mt-5 grid gap-5"
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const base = {
            ...requirementsFromForm(form),
            bypassActors: bypassActorsFromForm(form.get("bypassActors")),
            enforcement: String(
              form.get("enforcement") ?? "active",
            ) as BranchPolicyEnforcement,
          };

          if (isRule) {
            onSubmit({
              ...base,
              action: item ? "update-rule" : "create-rule",
              description: String(form.get("description") ?? ""),
              pattern: String(form.get("pattern") ?? ""),
              ruleId: item?.id,
            });
            return;
          }
          onSubmit({
            ...base,
            action: item ? "update-ruleset" : "create-ruleset",
            name: String(form.get("name") ?? ""),
            patterns: splitList(form.get("patterns")),
            rulesetId: item?.id,
          });
        }}
      >
        <div className="grid gap-4 lg:grid-cols-2">
          {isRule ? (
            <label className="grid gap-2" htmlFor="branch-rule-pattern">
              <span className="t-label">Branch pattern</span>
              <input
                className="input"
                defaultValue={item && "pattern" in item ? item.pattern : ""}
                id="branch-rule-pattern"
                name="pattern"
                placeholder={settings.defaultBranch}
                required
              />
            </label>
          ) : (
            <>
              <label className="grid gap-2" htmlFor="ruleset-name">
                <span className="t-label">Ruleset name</span>
                <input
                  className="input"
                  defaultValue={item && "name" in item ? item.name : ""}
                  id="ruleset-name"
                  name="name"
                  placeholder="Release branches"
                  required
                />
              </label>
              <label className="grid gap-2" htmlFor="ruleset-patterns">
                <span className="t-label">Branch patterns</span>
                <textarea
                  className="input min-h-24"
                  defaultValue={
                    item && "patterns" in item ? item.patterns.join("\n") : ""
                  }
                  id="ruleset-patterns"
                  name="patterns"
                  placeholder="main&#10;release/*"
                  required
                />
              </label>
            </>
          )}
          {isRule ? (
            <label className="grid gap-2" htmlFor="branch-rule-description">
              <span className="t-label">Description</span>
              <input
                className="input"
                defaultValue={
                  item && "description" in item ? (item.description ?? "") : ""
                }
                id="branch-rule-description"
                name="description"
                placeholder="Protect the release branch"
              />
            </label>
          ) : null}
        </div>

        <fieldset className="grid gap-2">
          <legend className="t-label">Enforcement status</legend>
          <div className="flex flex-wrap gap-2">
            {(["active", "evaluate", "disabled"] as const).map((value) => (
              <label className="chip soft" key={value}>
                <input
                  className="mr-2"
                  defaultChecked={(item?.enforcement ?? "active") === value}
                  name="enforcement"
                  type="radio"
                  value={value}
                />
                {enforcementLabel(value)}
              </label>
            ))}
          </div>
        </fieldset>

        <div className="grid gap-4 lg:grid-cols-2">
          <label className="grid gap-2" htmlFor="required-review-count">
            <span className="t-label">Required reviews</span>
            <input
              className="input"
              defaultValue={requirements.requiredApprovingReviewCount}
              id="required-review-count"
              min={0}
              name="requiredApprovingReviewCount"
              type="number"
            />
          </label>
          <label className="grid gap-2" htmlFor="required-status-checks">
            <span className="t-label">Required status checks</span>
            <textarea
              className="input min-h-24"
              defaultValue={requirements.requiredStatusChecks.join("\n")}
              id="required-status-checks"
              name="requiredStatusChecks"
              placeholder={settings.statusCheckSuggestions.join(", ") || "ci"}
            />
          </label>
        </div>

        <fieldset className="grid gap-3">
          <legend className="t-label">Requirements</legend>
          <div className="grid gap-3 sm:grid-cols-2">
            {checkboxLabel(
              "requiresUpToDateBranch",
              "Require branches to be up to date",
              requirements,
            )}
            {checkboxLabel(
              "requiresConversationResolution",
              "Require conversation resolution",
              requirements,
            )}
            {checkboxLabel(
              "requiresSignedCommits",
              "Require signed commits",
              requirements,
            )}
            {checkboxLabel(
              "requiresLinearHistory",
              "Require linear history",
              requirements,
            )}
            {checkboxLabel(
              "requiresMergeQueue",
              "Require merge queue",
              requirements,
            )}
            {checkboxLabel(
              "requiresDeployments",
              "Require deployments",
              requirements,
            )}
            {checkboxLabel("locked", "Lock branch", requirements)}
            {checkboxLabel("restrictsPushes", "Restrict pushes", requirements)}
            {checkboxLabel(
              "allowsForcePushes",
              "Allow force pushes",
              requirements,
            )}
            {checkboxLabel("allowsDeletions", "Allow deletions", requirements)}
          </div>
        </fieldset>

        <div className="grid gap-4 lg:grid-cols-2">
          <label className="grid gap-2" htmlFor="deployment-environments">
            <span className="t-label">Deployment environments</span>
            <textarea
              className="input min-h-20"
              defaultValue={requirements.requiredDeploymentEnvironments.join(
                "\n",
              )}
              id="deployment-environments"
              name="requiredDeploymentEnvironments"
              placeholder="production"
            />
          </label>
          <label className="grid gap-2" htmlFor="bypass-actors">
            <span className="t-label">Bypass actors</span>
            <textarea
              className="input min-h-20"
              defaultValue={bypassActorsToText(item?.bypassActors ?? [])}
              id="bypass-actors"
              name="bypassActors"
              placeholder="team:00000000-0000-0000-0000-000000000000:Core"
            />
          </label>
        </div>

        {error ? (
          <p className="t-sm" role="alert" style={{ color: "var(--err)" }}>
            {error}
          </p>
        ) : null}
        <div className="flex flex-wrap justify-end gap-2">
          <button className="btn" onClick={onCancel} type="button">
            Cancel
          </button>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Saving..." : item ? "Save policy" : "Create policy"}
          </button>
        </div>
      </form>
    </section>
  );
}

function PolicyIntentCard({
  intent,
  onStart,
}: {
  intent: "rule" | "ruleset";
  onStart: (mode: Exclude<EditorMode, null>) => void;
}) {
  return (
    <section className="card p-5" role="status">
      <span className="chip active">
        {intent === "rule" ? "New rule" : "New ruleset"}
      </span>
      <h2 className="t-h3 mt-3">
        {intent === "rule"
          ? "Branch protection rule editor"
          : "Repository ruleset editor"}
      </h2>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        Start a server-confirmed editor for branch patterns, requirements,
        bypass actors, and enforcement status.
      </p>
      <button
        className="btn primary mt-4"
        onClick={() => onStart({ kind: intent })}
        type="button"
      >
        Open editor
      </button>
    </section>
  );
}

function DefaultBranchCard({
  settings,
}: {
  settings: RepositoryBranchSettings;
}) {
  const summary = settings.defaultBranchSummary;
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Default branch
          </p>
          <h2 className="t-h2 mt-2">
            <Link className="hover:underline" href={summary.href}>
              {summary.name}
            </Link>
          </h2>
        </div>
        <span className={summary.protected ? "chip ok" : "chip soft"}>
          {summary.protected ? "Protected" : "Unprotected"}
        </span>
      </div>
      <div className="mt-4 grid gap-3 sm:grid-cols-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Matching rules
          </p>
          <p className="t-num mt-1 text-lg">{summary.matchingRuleCount}</p>
        </div>
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Matching rulesets
          </p>
          <p className="t-num mt-1 text-lg">{summary.matchingRulesetCount}</p>
        </div>
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Viewer
          </p>
          <p className="t-sm mt-1 capitalize">{settings.viewerPermission}</p>
        </div>
      </div>
    </section>
  );
}

function BranchRefsCard({ settings }: { settings: RepositoryBranchSettings }) {
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Branch refs
          </p>
          <h2 className="t-h3 mt-2">Protected branch coverage</h2>
        </div>
        <span className="chip soft">{settings.refs.length} branches</span>
      </div>
      <div className="mt-4 grid gap-2">
        {settings.refs.length === 0 ? (
          <p className="t-body" style={{ color: "var(--ink-2)" }}>
            No branch refs have been indexed for this repository yet.
          </p>
        ) : (
          settings.refs.slice(0, 6).map((branch) => (
            <div className="list-row" key={branch.name}>
              <div className="min-w-0 flex-1">
                <Link
                  className="t-mono-sm font-semibold hover:underline"
                  href={`/${settings.ownerLogin}/${settings.name}/tree/${branch.name}`}
                >
                  {branch.name}
                </Link>
                <p className="t-xs mt-1">
                  Updated {formatDate(branch.updatedAt)}
                </p>
              </div>
              <span className={branch.protected ? "chip ok" : "chip soft"}>
                {branch.protected ? "Protected" : "Open"}
              </span>
              <span className="chip soft">
                {branch.matchingRuleCount + branch.matchingRulesetCount} matches
              </span>
            </div>
          ))
        )}
      </div>
    </section>
  );
}

function PolicyCardView({
  card,
  onDelete,
  onEdit,
}: {
  card: PolicyCard;
  onDelete: (card: PolicyCard) => void;
  onEdit: (card: PolicyCard) => void;
}) {
  const requirements = requirementChips(card.item.requirements);
  const patterns = policyPatterns(card);
  return (
    <article
      className="list-row items-start"
      id={`${card.kind}-${card.item.id}`}
    >
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="t-h3 break-words">{policyTitle(card)}</h3>
          <span className={enforcementChipClass(card.item.enforcement)}>
            {enforcementLabel(card.item.enforcement)}
          </span>
          <span className="chip soft">
            {card.item.matchingBranchCount} matching branches
          </span>
        </div>
        {"description" in card.item && card.item.description ? (
          <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
            {card.item.description}
          </p>
        ) : null}
        <div className="mt-3 flex flex-wrap gap-2">
          {patterns.map((pattern) => (
            <span className="chip soft t-mono-sm" key={pattern}>
              {pattern}
            </span>
          ))}
        </div>
        <div className="mt-3 flex flex-wrap gap-2">
          {requirements.length > 0 ? (
            requirements.map((requirement) => (
              <span className="chip" key={requirement}>
                {requirement}
              </span>
            ))
          ) : (
            <span className="chip soft">No blocking requirements</span>
          )}
        </div>
        {card.item.bypassActors.length > 0 ? (
          <p className="t-xs mt-3">
            Bypass:{" "}
            {card.item.bypassActors.map((actor) => actor.label).join(", ")}
          </p>
        ) : null}
        {card.item.matchingBranches.length > 0 ? (
          <p className="t-xs mt-3">
            Matches {card.item.matchingBranches.slice(0, 4).join(", ")}
            {card.item.matchingBranches.length > 4 ? "..." : ""}
          </p>
        ) : null}
      </div>
      <div className="flex shrink-0 flex-col gap-2 text-right">
        <span className="t-xs">Updated {formatDate(card.item.updatedAt)}</span>
        {card.item.canEdit ? (
          <button className="btn sm" onClick={() => onEdit(card)} type="button">
            Edit
          </button>
        ) : (
          <span className="chip soft">Read-only</span>
        )}
        {card.item.canDelete ? (
          <button
            className="btn sm"
            onClick={() => onDelete(card)}
            type="button"
          >
            Delete
          </button>
        ) : null}
      </div>
    </article>
  );
}

function PolicyList({
  cards,
  settings,
  onDelete,
  onEdit,
  onNew,
}: {
  cards: PolicyCard[];
  settings: RepositoryBranchSettings;
  onDelete: (card: PolicyCard) => void;
  onEdit: (card: PolicyCard) => void;
  onNew: (mode: Exclude<EditorMode, null>) => void;
}) {
  return (
    <section className="card p-0" id="branch-rules">
      <div
        className="flex flex-wrap items-center justify-between gap-3 p-5"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Policies
          </p>
          <h2 className="t-h3 mt-2">Rules and rulesets</h2>
        </div>
        {settings.canEdit ? (
          <div className="flex flex-wrap gap-2">
            <button
              className="btn primary"
              onClick={() => onNew({ kind: "rule" })}
              type="button"
            >
              New branch protection rule
            </button>
            <button
              className="btn"
              onClick={() => onNew({ kind: "ruleset" })}
              type="button"
            >
              New ruleset
            </button>
          </div>
        ) : (
          <p className="t-sm" style={{ color: "var(--ink-2)" }}>
            You can view active and evaluate-only policies, but editing requires
            admin access.
          </p>
        )}
      </div>
      {cards.length === 0 ? (
        <div className="p-5">
          <span className="chip soft">No policies</span>
          <h3 className="t-h3 mt-3">No branch rules are configured</h3>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Add a branch protection rule or ruleset to explain and enforce merge
            and push requirements.
          </p>
          {settings.canEdit ? (
            <div className="mt-4 flex flex-wrap gap-2">
              <button
                className="btn primary"
                onClick={() => onNew({ kind: "rule" })}
                type="button"
              >
                New branch protection rule
              </button>
              <button
                className="btn"
                onClick={() => onNew({ kind: "ruleset" })}
                type="button"
              >
                New ruleset
              </button>
            </div>
          ) : null}
        </div>
      ) : (
        <div>
          {cards.map((card) => (
            <PolicyCardView
              card={card}
              key={`${card.kind}-${card.item.id}`}
              onDelete={onDelete}
              onEdit={onEdit}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function ReadOnlyExplanation({
  settings,
}: {
  settings: RepositoryBranchSettings;
}) {
  if (settings.canEdit) return null;
  return (
    <section className="card p-5">
      <span className="chip warn">Read-only</span>
      <h2 className="t-h3 mt-3">Policies can block pushes and merges</h2>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        Active rules are enforced. Evaluate-only rules are visible here so
        maintainers can explain future restrictions without blocking work yet.
      </p>
    </section>
  );
}

export function RepositoryBranchSettingsPage({
  intent,
  repository,
  settingsResult,
}: RepositoryBranchSettingsPageProps) {
  if (!settingsResult.ok) {
    return (
      <BranchSettingsUnavailable
        repository={repository}
        result={settingsResult}
      />
    );
  }

  return (
    <RepositoryBranchSettingsContent
      initialIntent={intent}
      initialSettings={settingsResult.settings}
      repository={repository}
    />
  );
}

function RepositoryBranchSettingsContent({
  initialIntent,
  initialSettings,
  repository,
}: {
  initialIntent?: "rule" | "ruleset";
  initialSettings: RepositoryBranchSettings;
  repository: RepositoryOverview;
}) {
  const [settings, setSettings] = useState(initialSettings);
  const [editor, setEditor] = useState<EditorMode>(null);
  const [confirmation, setConfirmation] = useState<Confirmation>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const actionUrl = `/${repository.owner_login}/${repository.name}/settings/branches/actions`;
  const cards: PolicyCard[] = useMemo(
    () => [
      ...settings.rules.map((item) => ({ kind: "rule" as const, item })),
      ...settings.rulesets.map((item) => ({
        kind: "ruleset" as const,
        item,
      })),
    ],
    [settings.rules, settings.rulesets],
  );

  async function mutate(mutation: RepositoryBranchPolicyMutation) {
    setBusy(true);
    setError(null);
    setNotice(null);
    const response = await fetch(actionUrl, {
      body: JSON.stringify(mutation),
      headers: { "content-type": "application/json" },
      method: "POST",
    });
    const body = await response.json().catch(() => null);
    setBusy(false);
    if (!response.ok) {
      setError(
        body?.error?.message ??
          "Repository branch policy update failed. Try again.",
      );
      return;
    }
    setSettings(body as RepositoryBranchSettings);
    setEditor(null);
    setConfirmation(null);
    setNotice("Branch policy saved.");
  }

  function startEdit(card: PolicyCard) {
    setError(null);
    setEditor(
      card.kind === "rule"
        ? { kind: "rule", item: card.item }
        : { kind: "ruleset", item: card.item },
    );
  }

  function startDelete(card: PolicyCard) {
    setError(null);
    setConfirmation(
      card.kind === "rule"
        ? { kind: "rule", item: card.item }
        : { kind: "ruleset", item: card.item },
    );
  }

  return (
    <div className="grid gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Branch policy
          </p>
          <h2 className="t-h2 mt-2">
            {settings.ownerLogin}/{settings.name}
          </h2>
        </div>
        <div className="flex flex-wrap gap-2">
          <span className="chip soft">Viewer: {settings.viewerPermission}</span>
          <span className={settings.canEdit ? "chip ok" : "chip warn"}>
            {settings.canEdit ? "Editable" : "Read-only"}
          </span>
        </div>
      </div>

      {notice ? (
        <p className="chip ok w-fit" role="status">
          {notice}
        </p>
      ) : null}

      {initialIntent && !editor ? (
        <PolicyIntentCard
          intent={initialIntent}
          onStart={(mode) => {
            setError(null);
            setEditor(mode);
          }}
        />
      ) : null}

      {editor ? (
        <PolicyEditor
          busy={busy}
          error={error}
          mode={editor}
          onCancel={() => {
            setEditor(null);
            setError(null);
          }}
          onSubmit={(mutation) => void mutate(mutation)}
          settings={settings}
        />
      ) : null}

      {confirmation ? (
        <section className="card p-5" role="alertdialog">
          <span className="chip warn">Confirm deletion</span>
          <h2 className="t-h3 mt-3">
            Delete {confirmation.kind === "rule" ? "rule" : "ruleset"}{" "}
            {confirmation.kind === "rule"
              ? confirmation.item.pattern
              : confirmation.item.name}
          </h2>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            The policy list refreshes only after the API accepts the delete.
          </p>
          {error ? (
            <p
              className="t-sm mt-3"
              role="alert"
              style={{ color: "var(--err)" }}
            >
              {error}
            </p>
          ) : null}
          <div className="mt-5 flex flex-wrap justify-end gap-2">
            <button
              className="btn"
              onClick={() => {
                setConfirmation(null);
                setError(null);
              }}
              type="button"
            >
              Keep policy
            </button>
            <button
              className="btn primary"
              disabled={busy}
              onClick={() =>
                void mutate(
                  confirmation.kind === "rule"
                    ? { action: "delete-rule", ruleId: confirmation.item.id }
                    : {
                        action: "delete-ruleset",
                        rulesetId: confirmation.item.id,
                      },
                )
              }
              type="button"
            >
              {busy ? "Deleting..." : "Delete policy"}
            </button>
          </div>
        </section>
      ) : null}

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="grid gap-5">
          <DefaultBranchCard settings={settings} />
          <ReadOnlyExplanation settings={settings} />
          <PolicyList
            cards={cards}
            onDelete={startDelete}
            onEdit={startEdit}
            onNew={(mode) => {
              setError(null);
              setEditor(mode);
            }}
            settings={settings}
          />
        </div>
        <div className="grid content-start gap-5">
          <BranchRefsCard settings={settings} />
          <section className="card p-5">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Status checks
            </p>
            <h2 className="t-h3 mt-2">Known contexts</h2>
            <div className="mt-4 flex flex-wrap gap-2">
              {settings.statusCheckSuggestions.length > 0 ? (
                settings.statusCheckSuggestions.map((context) => (
                  <span className="chip soft t-mono-sm" key={context}>
                    {context}
                  </span>
                ))
              ) : (
                <span className="chip soft">No suggestions yet</span>
              )}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
