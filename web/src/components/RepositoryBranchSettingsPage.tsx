import Link from "next/link";
import type {
  BranchPolicyEnforcement,
  BranchPolicyRequirements,
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

function PolicyIntentCard({
  intent,
  repository,
}: {
  intent: "rule" | "ruleset";
  repository: RepositoryOverview;
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
        The write contract is available now. The full editor, validation flow,
        and server-confirmed saves are the next vertical slice.
      </p>
      <Link
        className="btn mt-4"
        href={`/${repository.owner_login}/${repository.name}/settings/branches`}
      >
        Return to policies
      </Link>
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

function PolicyCardView({ card }: { card: PolicyCard }) {
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
          <Link
            className="btn sm"
            href={`?edit=${card.item.id}#${card.kind}-${card.item.id}`}
          >
            Edit
          </Link>
        ) : (
          <span className="chip soft">Read-only</span>
        )}
      </div>
    </article>
  );
}

function PolicyList({
  cards,
  repository,
  settings,
}: {
  cards: PolicyCard[];
  repository: RepositoryOverview;
  settings: RepositoryBranchSettings;
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
            <Link
              className="btn primary"
              href={`/${repository.owner_login}/${repository.name}/settings/branches?new=rule#branch-rules`}
            >
              New branch protection rule
            </Link>
            <Link
              className="btn"
              href={`/${repository.owner_login}/${repository.name}/settings/branches?new=ruleset#branch-rules`}
            >
              New ruleset
            </Link>
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
              <Link
                className="btn primary"
                href={`/${repository.owner_login}/${repository.name}/settings/branches?new=rule#branch-rules`}
              >
                New branch protection rule
              </Link>
              <Link
                className="btn"
                href={`/${repository.owner_login}/${repository.name}/settings/branches?new=ruleset#branch-rules`}
              >
                New ruleset
              </Link>
            </div>
          ) : null}
        </div>
      ) : (
        <div>
          {cards.map((card) => (
            <PolicyCardView card={card} key={`${card.kind}-${card.item.id}`} />
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

  const settings = settingsResult.settings;
  const cards: PolicyCard[] = [
    ...settings.rules.map((item) => ({ kind: "rule" as const, item })),
    ...settings.rulesets.map((item) => ({ kind: "ruleset" as const, item })),
  ];

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

      {intent ? (
        <PolicyIntentCard intent={intent} repository={repository} />
      ) : null}

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="grid gap-5">
          <DefaultBranchCard settings={settings} />
          <ReadOnlyExplanation settings={settings} />
          <PolicyList
            cards={cards}
            repository={repository}
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
