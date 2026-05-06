"use client";

import Link from "next/link";
import type { ReactNode } from "react";
import { useState } from "react";
import { LabelPicker } from "@/components/LabelPicker";
import { MarkdownBody } from "@/components/MarkdownBody";
import { MilestonePicker } from "@/components/MilestonePicker";
import { PullRequestTimeline } from "@/components/PullRequestTimeline";
import { RepositoryShell } from "@/components/RepositoryShell";
import { ThreadNotificationCard } from "@/components/ThreadNotificationCard";
import type {
  ApiErrorEnvelope,
  IssueListLabel,
  IssueListMilestone,
  IssueListUser,
  MergeMethod,
  PullRequestDetailView,
  PullRequestSubscriptionState,
  PullRequestTimelineItem,
  RepositoryOverview,
  ThreadSubscriptionEvent,
} from "@/lib/api";

type RepositoryPullRequestDetailPageProps = {
  repository: RepositoryOverview;
  pullRequest: PullRequestDetailView;
  timeline: PullRequestTimelineItem[];
  viewerAuthenticated: boolean;
};

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) {
    return "just now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  const days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days}d ago`;
  }
  const months = Math.floor(days / 30);
  return months < 12 ? `${months}mo ago` : `${Math.floor(months / 12)}y ago`;
}

function avatarLabel(login: string) {
  return login.slice(0, 1).toUpperCase();
}

function optionSelected(id: string, selectedIds: string[]) {
  return selectedIds.includes(id);
}

function stateLabel(pullRequest: PullRequestDetailView) {
  if (pullRequest.state === "merged") {
    return "Merged";
  }
  if (pullRequest.state === "closed") {
    return "Closed";
  }
  return pullRequest.isDraft ? "Draft" : "Open";
}

function stateClass(pullRequest: PullRequestDetailView) {
  if (pullRequest.state === "merged") {
    return "accent";
  }
  if (pullRequest.state === "closed") {
    return "err";
  }
  return pullRequest.isDraft ? "warn" : "ok";
}

function mergeMethodLabel(method: MergeMethod) {
  if (method === "merge_commit") {
    return "Create a merge commit";
  }
  if (method === "rebase") {
    return "Rebase and merge";
  }
  return "Squash and merge";
}

function mergeMethodHelp(method: MergeMethod) {
  if (method === "merge_commit") {
    return "Preserves the branch history and adds one merge commit to the base branch.";
  }
  if (method === "rebase") {
    return "Replays each head commit onto the base branch without a merge commit.";
  }
  return "Combines the pull request into one commit on the base branch.";
}

function defaultCommitTitle(
  method: MergeMethod,
  pullRequest: PullRequestDetailView,
) {
  if (method === "merge_commit") {
    return `Merge pull request #${pullRequest.number} from ${pullRequest.headRef}`;
  }
  if (method === "rebase") {
    return `Rebase pull request #${pullRequest.number} onto ${pullRequest.baseRef}`;
  }
  return `${pullRequest.title} (#${pullRequest.number})`;
}

function blockerDetails(envelope: ApiErrorEnvelope | null) {
  const blockers = envelope?.details?.blockers;
  if (!Array.isArray(blockers)) {
    return [];
  }
  return blockers
    .map((blocker) => {
      if (
        blocker !== null &&
        typeof blocker === "object" &&
        "message" in blocker &&
        typeof blocker.message === "string"
      ) {
        return blocker.message;
      }
      return null;
    })
    .filter((message): message is string => message !== null);
}

function SidebarSection({
  children,
  title,
}: {
  children: ReactNode;
  title: string;
}) {
  return (
    <section className="border-b py-4" style={{ borderColor: "var(--line)" }}>
      <h2 className="t-label mb-3">{title}</h2>
      {children}
    </section>
  );
}

export function RepositoryPullRequestDetailPage({
  repository,
  pullRequest: initialPullRequest,
  timeline,
  viewerAuthenticated,
}: RepositoryPullRequestDetailPageProps) {
  const [currentPullRequest, setCurrentPullRequest] =
    useState(initialPullRequest);
  const [subscription, setSubscription] = useState(
    initialPullRequest.subscription,
  );
  const [message, setMessage] = useState<string | null>(null);
  const [isMutating, setIsMutating] = useState(false);
  const [openMetadataMenu, setOpenMetadataMenu] = useState<
    "reviewers" | "assignees" | "labels" | "milestone" | null
  >(null);
  const [mergeMethod, setMergeMethod] = useState<MergeMethod>(
    initialPullRequest.mergeability.defaultMethod,
  );
  const [commitTitle, setCommitTitle] = useState(() =>
    defaultCommitTitle(
      initialPullRequest.mergeability.defaultMethod,
      initialPullRequest,
    ),
  );
  const [commitBody, setCommitBody] = useState("");
  const [deleteBranch, setDeleteBranch] = useState(false);
  const [mergeConfirmOpen, setMergeConfirmOpen] = useState(false);
  const [mergeBlockers, setMergeBlockers] = useState<string[]>([]);
  const pullRequest = currentPullRequest;
  const branchProtection = pullRequest.mergeability.branchProtection ?? {
    protected: false,
    pattern: null,
    requiredApprovingReviewCount: 0,
    requiresUpToDateBranch: false,
    requiredStatusChecks: [],
    requiredDeploymentEnvironments: [],
  };
  const bodyLabelId = `pull-request-${pullRequest.number}-body`;
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const activePath = `${basePath}/pull/${pullRequest.number}`;
  const rawDiffHref = `/api/repos/${encodeURIComponent(repository.owner_login)}/${encodeURIComponent(repository.name)}/pulls/${pullRequest.number}.diff`;
  const rawPatchHref = `/api/repos/${encodeURIComponent(repository.owner_login)}/${encodeURIComponent(repository.name)}/pulls/${pullRequest.number}.patch`;
  const canEditMetadata =
    viewerAuthenticated && currentPullRequest.viewerPermission !== null;
  const canDeleteHeadBranch =
    pullRequest.mergeability.canMerge &&
    pullRequest.headRef !== pullRequest.baseRef &&
    !pullRequest.headRef.startsWith(`${repository.owner_login}:`);
  const tabItems = [
    {
      href: activePath,
      label: "Conversation",
      count: currentPullRequest.stats.comments,
    },
    {
      href: `${activePath}/commits`,
      label: "Commits",
      count: currentPullRequest.stats.commits,
    },
    {
      href: `${activePath}/checks`,
      label: "Checks",
      count: currentPullRequest.checks.totalCount || null,
    },
    {
      href: currentPullRequest.filesHref,
      label: "Files changed",
      count: currentPullRequest.stats.files,
    },
  ];

  async function savePullRequest(
    path: "metadata" | "review-requests" | "draft",
    body: Record<string, unknown>,
    success: string,
  ) {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/${path}`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Pull request could not be updated.",
        );
      }
      const updated = payload as PullRequestDetailView;
      setCurrentPullRequest(updated);
      setSubscription(updated.subscription);
      setOpenMetadataMenu(null);
      setMessage(success);
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Pull request could not be updated.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  async function saveState(state: "open" | "closed", success: string) {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/state`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ state }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Pull request state could not be updated.",
        );
      }
      const updated = payload as PullRequestDetailView;
      setCurrentPullRequest(updated);
      setSubscription(updated.subscription);
      setMergeMethod(updated.mergeability.defaultMethod);
      setCommitTitle(
        defaultCommitTitle(updated.mergeability.defaultMethod, updated),
      );
      setCommitBody("");
      setDeleteBranch(false);
      setMergeConfirmOpen(false);
      setMergeBlockers([]);
      setMessage(success);
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Pull request state could not be updated.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  async function mergePullRequest() {
    setMessage(null);
    setMergeBlockers([]);
    if (!commitTitle.trim()) {
      setMergeBlockers(["Commit title is required before merging."]);
      return;
    }
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/merge`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          method: mergeMethod,
          commitTitle: commitTitle.trim(),
          commitBody: commitBody.trim() || null,
          deleteBranch: canDeleteHeadBranch && deleteBranch,
        }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        const details = blockerDetails(envelope);
        if (details.length) {
          setMergeBlockers(details);
        }
        throw new Error(
          envelope?.error.message ?? "Pull request could not merge.",
        );
      }
      const updated = payload as PullRequestDetailView;
      setCurrentPullRequest(updated);
      setSubscription(updated.subscription);
      setMergeConfirmOpen(false);
      setDeleteBranch(false);
      setMergeBlockers([]);
      setMessage("Pull request merged.");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Pull request could not merge.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  function selectMergeMethod(method: MergeMethod) {
    setMergeMethod(method);
    setCommitTitle(defaultCommitTitle(method, pullRequest));
    setMergeBlockers([]);
  }

  function saveMetadata(next: {
    labels?: IssueListLabel[];
    assignees?: IssueListUser[];
    milestone?: IssueListMilestone | null;
  }) {
    const labels = next.labels ?? currentPullRequest.labels;
    const assignees = next.assignees ?? currentPullRequest.assignees;
    const milestone =
      "milestone" in next ? next.milestone : currentPullRequest.milestone;
    void savePullRequest(
      "metadata",
      {
        labelIds: labels.map((label) => label.id),
        assigneeUserIds: assignees.map((assignee) => assignee.id),
        milestoneId: milestone?.id ?? null,
      },
      "Pull request metadata updated.",
    );
  }

  function toggleReviewer(reviewer: IssueListUser) {
    const selectedIds = currentPullRequest.requestedReviewers.map(
      (item) => item.id,
    );
    const reviewerUserIds = optionSelected(reviewer.id, selectedIds)
      ? selectedIds.filter((id) => id !== reviewer.id)
      : [...selectedIds, reviewer.id];
    void savePullRequest(
      "review-requests",
      { reviewerUserIds },
      "Review requests updated.",
    );
  }

  function toggleAssignee(assignee: IssueListUser) {
    const selectedIds = currentPullRequest.assignees.map((item) => item.id);
    const assignees = optionSelected(assignee.id, selectedIds)
      ? currentPullRequest.assignees.filter((item) => item.id !== assignee.id)
      : [...currentPullRequest.assignees, assignee];
    saveMetadata({ assignees });
  }

  function saveLabels(labels: IssueListLabel[]) {
    saveMetadata({ labels });
  }

  function toggleDraft() {
    void savePullRequest(
      "draft",
      { isDraft: !currentPullRequest.isDraft },
      currentPullRequest.isDraft
        ? "Pull request marked ready for review."
        : "Pull request converted to draft.",
    );
  }

  async function saveSubscription(
    subscribed: boolean,
    customEvents: ThreadSubscriptionEvent[],
  ) {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/subscription`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ subscribed, customEvents }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ??
            "Notification subscription could not be updated.",
        );
      }
      const next = payload as PullRequestSubscriptionState;
      setSubscription(next);
      setMessage(
        next.subscribed ? "Subscribed to notifications." : "Unsubscribed.",
      );
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Notification subscription could not be updated.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <RepositoryShell
      activePath={`${basePath}/pulls`}
      frameClassName="max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0">
        <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <h2 className="t-label mr-2">
                Pull request #{pullRequest.number}
              </h2>
              <Link className="btn sm" href={`${basePath}/pulls`}>
                All pull requests
              </Link>
              <Link className="btn primary sm" href={`${basePath}/compare`}>
                New pull request
              </Link>
            </div>
            <h1 className="t-h1 break-words">
              {pullRequest.title}{" "}
              <span className="t-num" style={{ color: "var(--ink-4)" }}>
                #{pullRequest.number}
              </span>
            </h1>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <span className={`chip ${stateClass(pullRequest)}`}>
                {stateLabel(pullRequest)}
              </span>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                <span className="av sm mr-1 inline-flex align-middle">
                  {avatarLabel(pullRequest.author.login)}
                </span>
                <strong style={{ color: "var(--ink-1)" }}>
                  {pullRequest.author.login}
                </strong>{" "}
                wants to merge{" "}
                <span className="t-num">{pullRequest.stats.commits}</span>{" "}
                {pullRequest.stats.commits === 1 ? "commit" : "commits"} into{" "}
                <span className="t-mono-sm">{pullRequest.baseRef}</span> from{" "}
                <span className="t-mono-sm">{pullRequest.headRef}</span>
              </p>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {canEditMetadata ? (
              <button
                className="btn"
                disabled={isMutating}
                onClick={() => toggleDraft()}
                type="button"
              >
                {pullRequest.isDraft ? "Mark ready" : "Convert to draft"}
              </button>
            ) : null}
            <Link className="btn" href={pullRequest.filesHref}>
              View changes
            </Link>
            <Link className="btn ghost" href={rawDiffHref}>
              .diff
            </Link>
            <Link className="btn ghost" href={rawPatchHref}>
              .patch
            </Link>
            <Link className="btn ghost" href="/docs/api#pulls-raw-diff">
              API docs
            </Link>
          </div>
        </div>

        {message ? (
          <div aria-live="polite" className="card mb-4 p-3">
            <p className="t-sm">{message}</p>
          </div>
        ) : null}

        <nav aria-label="Pull request sections" className="tabs mb-6">
          {tabItems.map((item) => (
            <Link
              aria-current={item.href === activePath ? "page" : undefined}
              className={`tab ${item.href === activePath ? "active" : ""}`}
              href={item.href}
              key={item.label}
            >
              {item.label}
              {item.count !== null ? (
                <span className="badge t-num">{item.count}</span>
              ) : null}
            </Link>
          ))}
        </nav>

        <div className="grid grid-cols-[minmax(0,1fr)_296px] gap-8 max-lg:grid-cols-1">
          <div className="min-w-0">
            <article className="flex gap-4">
              <div className="av lg shrink-0" aria-hidden="true">
                {avatarLabel(pullRequest.author.login)}
              </div>
              <div className="card min-w-0 flex-1 overflow-hidden">
                <div
                  className="flex flex-wrap items-center gap-2 border-b px-4 py-3"
                  style={{
                    background: "var(--surface-2)",
                    borderColor: "var(--line)",
                  }}
                >
                  <h2 className="t-sm font-semibold" id={bodyLabelId}>
                    {pullRequest.author.login}
                  </h2>
                  <span className="t-xs">
                    opened {relativeTime(pullRequest.createdAt)}
                  </span>
                  <span className="chip soft ml-auto">
                    {pullRequest.authorRole}
                  </span>
                </div>
                <div className="p-5">
                  {pullRequest.body?.trim() ? (
                    <MarkdownBody
                      html={pullRequest.bodyHtml}
                      labelledBy={bodyLabelId}
                    />
                  ) : (
                    <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                      No description provided.
                    </p>
                  )}
                </div>
              </div>
            </article>

            <div className="mt-6">
              <PullRequestTimeline
                initialItems={timeline}
                loginHref={`/login?next=${encodeURIComponent(activePath)}`}
                owner={repository.owner_login}
                pullNumber={pullRequest.number}
                repo={repository.name}
                viewerAuthenticated={viewerAuthenticated}
              />
            </div>

            <section className="card mt-6 overflow-hidden">
              <div
                className="flex flex-wrap items-start gap-4 border-b px-5 py-4"
                style={{ borderColor: "var(--line)" }}
              >
                <div className="min-w-0 flex-1">
                  <h2 className="t-h3">Merge readiness</h2>
                  <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                    {pullRequest.mergeability.summary}
                  </p>
                </div>
                <span
                  className={`chip ${
                    pullRequest.mergeability.canMerge
                      ? "ok"
                      : pullRequest.state === "closed"
                        ? "err"
                        : pullRequest.state === "merged"
                          ? "accent"
                          : "warn"
                  }`}
                >
                  {pullRequest.mergeability.state.replaceAll("_", " ")}
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-3 px-5 py-4">
                <span
                  className={`chip ${pullRequest.checks.failedCount ? "err" : "soft"}`}
                >
                  {pullRequest.checks.failedCount
                    ? `${pullRequest.checks.failedCount} failed`
                    : pullRequest.checks.status.replaceAll("_", " ")}
                </span>
                <span className="chip soft">
                  {pullRequest.review.state.replaceAll("_", " ")}
                </span>
                <span className="chip soft">
                  <span className="t-num">{pullRequest.stats.additions}</span>{" "}
                  additions
                </span>
                <span className="chip soft">
                  <span className="t-num">{pullRequest.stats.deletions}</span>{" "}
                  deletions
                </span>
                {branchProtection.protected ? (
                  <span className="chip warn">
                    Protected branch{" "}
                    <span className="t-mono-sm">
                      {branchProtection.pattern}
                    </span>
                  </span>
                ) : null}
              </div>
              {branchProtection.protected ? (
                <div
                  className="border-t px-5 py-4"
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  <h3 className="t-label mb-2">Branch rules</h3>
                  <div className="flex flex-wrap gap-2">
                    {branchProtection.requiredApprovingReviewCount ? (
                      <span className="chip soft">
                        <span className="t-num">
                          {branchProtection.requiredApprovingReviewCount}
                        </span>{" "}
                        approving review required
                      </span>
                    ) : null}
                    {branchProtection.requiredStatusChecks.length ? (
                      <span className="chip soft">
                        Required checks:{" "}
                        {branchProtection.requiredStatusChecks.join(", ")}
                      </span>
                    ) : null}
                    {branchProtection.requiresUpToDateBranch ? (
                      <span className="chip soft">
                        Up-to-date branch required
                      </span>
                    ) : null}
                    {branchProtection.requiresLinearHistory ? (
                      <span className="chip soft">Linear history required</span>
                    ) : null}
                    {branchProtection.requiresSignedCommits ? (
                      <span className="chip soft">Signed commits required</span>
                    ) : null}
                    {branchProtection.requiresConversationResolution ? (
                      <span className="chip soft">
                        Conversations must be resolved
                      </span>
                    ) : null}
                    {branchProtection.requiresMergeQueue ? (
                      <span className="chip soft">Merge queue required</span>
                    ) : null}
                    {branchProtection.requiresDeployments ? (
                      <span className="chip soft">
                        Deployments required
                        {branchProtection.requiredDeploymentEnvironments?.length
                          ? `: ${branchProtection.requiredDeploymentEnvironments.join(", ")}`
                          : ""}
                      </span>
                    ) : null}
                    {branchProtection.locked ? (
                      <span className="chip err">Branch locked</span>
                    ) : null}
                    {(branchProtection.activeRuleCount ?? 0) +
                      (branchProtection.activeRulesetCount ?? 0) >
                    1 ? (
                      <span className="chip info">
                        {(branchProtection.activeRuleCount ?? 0) +
                          (branchProtection.activeRulesetCount ?? 0)}{" "}
                        policies combined
                      </span>
                    ) : null}
                  </div>
                </div>
              ) : null}
              {pullRequest.mergeability.blockers.length ? (
                <div
                  className="border-t px-5 py-4"
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  <h3 className="t-label mb-2">Blocking reasons</h3>
                  <ul className="grid gap-2">
                    {pullRequest.mergeability.blockers.map((blocker) => (
                      <li className="t-sm" key={blocker.code}>
                        {blocker.message}
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {viewerAuthenticated ? (
                <>
                  <div
                    className="flex flex-wrap items-center gap-2 border-t px-5 py-4"
                    style={{ borderColor: "var(--line-soft)" }}
                  >
                    {pullRequest.mergeability.methods.map((method) => (
                      <button
                        aria-pressed={mergeMethod === method}
                        className={`chip soft ${mergeMethod === method ? "active" : ""}`}
                        disabled={
                          isMutating || !pullRequest.mergeability.canMerge
                        }
                        key={method}
                        onClick={() => selectMergeMethod(method)}
                        type="button"
                      >
                        {mergeMethodLabel(method)}
                      </button>
                    ))}
                    <span className="flex-1" />
                    {pullRequest.mergeability.canMarkReady ? (
                      <button
                        className="btn"
                        disabled={isMutating}
                        onClick={() => toggleDraft()}
                        type="button"
                      >
                        Mark ready
                      </button>
                    ) : null}
                    {pullRequest.mergeability.canReopen ? (
                      <button
                        className="btn"
                        disabled={isMutating}
                        onClick={() =>
                          void saveState("open", "Pull request reopened.")
                        }
                        type="button"
                      >
                        Reopen pull request
                      </button>
                    ) : null}
                    {pullRequest.mergeability.canClose ? (
                      <button
                        className="btn"
                        disabled={isMutating}
                        onClick={() =>
                          void saveState("closed", "Pull request closed.")
                        }
                        type="button"
                      >
                        Close pull request
                      </button>
                    ) : null}
                    <button
                      aria-expanded={mergeConfirmOpen}
                      aria-label="Open merge confirmation"
                      className="btn accent"
                      disabled={
                        isMutating || !pullRequest.mergeability.canMerge
                      }
                      onClick={() => {
                        setMergeConfirmOpen(true);
                        setMergeBlockers([]);
                      }}
                      type="button"
                    >
                      {mergeMethodLabel(mergeMethod)}
                    </button>
                  </div>
                  {mergeConfirmOpen ? (
                    <div
                      className="border-t px-5 py-5"
                      style={{
                        background: "var(--surface-2)",
                        borderColor: "var(--line-soft)",
                      }}
                    >
                      <div className="mb-4 flex flex-wrap items-start justify-between gap-3">
                        <div>
                          <h3 className="t-h3">Confirm merge</h3>
                          <p
                            className="t-sm mt-1"
                            style={{ color: "var(--ink-3)" }}
                          >
                            {mergeMethodHelp(mergeMethod)}
                          </p>
                        </div>
                        <span className="chip ok">
                          {pullRequest.checks.completedCount} /{" "}
                          {pullRequest.checks.totalCount} checks complete
                        </span>
                      </div>
                      <div className="grid gap-4">
                        <label className="block" htmlFor="merge-commit-title">
                          <span className="t-label">Commit title</span>
                          <input
                            className="input mt-2 h-10 w-full px-3 t-sm"
                            disabled={isMutating}
                            id="merge-commit-title"
                            onChange={(event) =>
                              setCommitTitle(event.target.value)
                            }
                            value={commitTitle}
                          />
                        </label>
                        <label className="block" htmlFor="merge-commit-body">
                          <span className="t-label">Commit body</span>
                          <textarea
                            className="input mt-2 min-h-24 w-full resize-y p-3 t-sm"
                            disabled={isMutating}
                            id="merge-commit-body"
                            onChange={(event) =>
                              setCommitBody(event.target.value)
                            }
                            placeholder="Optional context for the merge commit"
                            value={commitBody}
                          />
                        </label>
                        <label className="flex items-start gap-3 t-sm">
                          <input
                            checked={deleteBranch}
                            className="mt-1"
                            disabled={isMutating || !canDeleteHeadBranch}
                            onChange={(event) =>
                              setDeleteBranch(event.target.checked)
                            }
                            type="checkbox"
                          />
                          <span>
                            Delete head branch after merge
                            <span
                              className="block t-xs"
                              style={{ color: "var(--ink-3)" }}
                            >
                              {canDeleteHeadBranch
                                ? `Removes ${pullRequest.headRef} after the base ref updates.`
                                : "Branch deletion is unavailable for this pull request."}
                            </span>
                          </span>
                        </label>
                      </div>
                      {mergeBlockers.length ? (
                        <div
                          aria-live="polite"
                          className="card mt-4 p-3"
                          style={{ background: "var(--err-soft)" }}
                        >
                          <h4 className="t-label mb-2">Merge blocked</h4>
                          <ul className="grid gap-1">
                            {mergeBlockers.map((blocker) => (
                              <li className="t-sm" key={blocker}>
                                {blocker}
                              </li>
                            ))}
                          </ul>
                        </div>
                      ) : null}
                      <div className="mt-4 flex flex-wrap justify-end gap-2">
                        <button
                          className="btn"
                          disabled={isMutating}
                          onClick={() => {
                            setMergeConfirmOpen(false);
                            setMergeBlockers([]);
                          }}
                          type="button"
                        >
                          Cancel
                        </button>
                        <button
                          className="btn accent"
                          disabled={isMutating || !commitTitle.trim()}
                          onClick={() => void mergePullRequest()}
                          type="button"
                        >
                          Confirm {mergeMethodLabel(mergeMethod)}
                        </button>
                      </div>
                    </div>
                  ) : null}
                </>
              ) : null}
            </section>

            {!viewerAuthenticated ? (
              <div className="card mt-6 p-5">
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  Sign in to comment, request reviews, or change merge state.
                </p>
                <Link
                  className="btn accent mt-3"
                  href={`/login?next=${encodeURIComponent(activePath)}`}
                >
                  Sign in to participate
                </Link>
              </div>
            ) : null}
          </div>

          <aside className="min-w-0">
            <SidebarSection title="Reviewers">
              <div className="mb-3 flex items-center justify-between gap-2">
                <span className="t-xs">People asked to review changes.</span>
                {canEditMetadata ? (
                  <button
                    aria-expanded={openMetadataMenu === "reviewers"}
                    className="btn sm"
                    disabled={isMutating}
                    onClick={() =>
                      setOpenMetadataMenu(
                        openMetadataMenu === "reviewers" ? null : "reviewers",
                      )
                    }
                    type="button"
                  >
                    Edit
                  </button>
                ) : null}
              </div>
              {openMetadataMenu === "reviewers" ? (
                <div className="card mb-3 p-2" role="menu">
                  {pullRequest.metadataOptions.assignees.length ? (
                    pullRequest.metadataOptions.assignees.map((reviewer) => {
                      const selected = pullRequest.requestedReviewers.some(
                        (item) => item.id === reviewer.id,
                      );
                      return (
                        <button
                          aria-pressed={selected}
                          className="btn ghost sm w-full justify-start"
                          key={reviewer.id}
                          onClick={() => toggleReviewer(reviewer)}
                          type="button"
                        >
                          <span className="av sm" aria-hidden="true">
                            {avatarLabel(reviewer.login)}
                          </span>
                          {selected ? "Remove" : "Request"} {reviewer.login}
                        </button>
                      );
                    })
                  ) : (
                    <p className="t-xs p-2">No reviewers available.</p>
                  )}
                </div>
              ) : null}
              {pullRequest.latestReviews.length ? (
                <div className="flex flex-col gap-2">
                  {pullRequest.latestReviews.map((review) => (
                    <div
                      className="flex items-center gap-2"
                      key={review.reviewer.id}
                    >
                      <span className="av sm" aria-hidden="true">
                        {avatarLabel(review.reviewer.login)}
                      </span>
                      <span className="t-sm flex-1">
                        {review.reviewer.login}
                      </span>
                      <span className="chip soft">
                        {review.state.replaceAll("_", " ")}
                      </span>
                    </div>
                  ))}
                </div>
              ) : pullRequest.requestedReviewers.length ? (
                <div className="flex flex-col gap-2">
                  {pullRequest.requestedReviewers.map((reviewer) => (
                    <div className="flex items-center gap-2" key={reviewer.id}>
                      <span className="av sm" aria-hidden="true">
                        {avatarLabel(reviewer.login)}
                      </span>
                      <span className="t-sm flex-1">{reviewer.login}</span>
                      <span className="chip soft">requested</span>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No review requests</p>
              )}
            </SidebarSection>

            <SidebarSection title="Assignees">
              <div className="mb-3 flex items-center justify-between gap-2">
                <span className="t-xs">People responsible for this PR.</span>
                {canEditMetadata ? (
                  <button
                    aria-expanded={openMetadataMenu === "assignees"}
                    className="btn sm"
                    disabled={isMutating}
                    onClick={() =>
                      setOpenMetadataMenu(
                        openMetadataMenu === "assignees" ? null : "assignees",
                      )
                    }
                    type="button"
                  >
                    Edit
                  </button>
                ) : null}
              </div>
              {openMetadataMenu === "assignees" ? (
                <div className="card mb-3 p-2" role="menu">
                  {pullRequest.metadataOptions.assignees.length ? (
                    pullRequest.metadataOptions.assignees.map((assignee) => {
                      const selected = pullRequest.assignees.some(
                        (item) => item.id === assignee.id,
                      );
                      return (
                        <button
                          aria-pressed={selected}
                          className="btn ghost sm w-full justify-start"
                          key={assignee.id}
                          onClick={() => toggleAssignee(assignee)}
                          type="button"
                        >
                          <span className="av sm" aria-hidden="true">
                            {avatarLabel(assignee.login)}
                          </span>
                          {selected ? "Remove" : "Assign"} {assignee.login}
                        </button>
                      );
                    })
                  ) : (
                    <p className="t-xs p-2">No assignable collaborators.</p>
                  )}
                </div>
              ) : null}
              {pullRequest.assignees.length ? (
                <div className="flex flex-col gap-2">
                  {pullRequest.assignees.map((assignee) => (
                    <div className="row gap-2" key={assignee.id}>
                      <span className="av sm" aria-hidden="true">
                        {avatarLabel(assignee.login)}
                      </span>
                      <span className="t-sm">{assignee.login}</span>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No one assigned</p>
              )}
            </SidebarSection>

            <SidebarSection title="Labels">
              <div className="mb-3 flex items-center justify-between gap-2">
                <span className="t-xs">Classify the pull request.</span>
                {canEditMetadata ? (
                  <button
                    aria-expanded={openMetadataMenu === "labels"}
                    className="btn sm"
                    disabled={isMutating}
                    onClick={() =>
                      setOpenMetadataMenu(
                        openMetadataMenu === "labels" ? null : "labels",
                      )
                    }
                    type="button"
                  >
                    Edit
                  </button>
                ) : null}
              </div>
              {openMetadataMenu === "labels" ? (
                <LabelPicker
                  disabled={isMutating}
                  labels={pullRequest.metadataOptions.labels}
                  onCancel={() => setOpenMetadataMenu(null)}
                  onSave={saveLabels}
                  selectedLabels={pullRequest.labels}
                  title="Pull request label picker"
                />
              ) : null}
              {pullRequest.labels.length ? (
                <div className="flex flex-wrap gap-2">
                  {pullRequest.labels.map((label) => (
                    <span
                      className="chip soft"
                      key={label.id}
                      title={label.description ?? label.name}
                    >
                      <span
                        aria-hidden="true"
                        className="inline-block h-2 w-2 rounded-full"
                        style={{ background: label.color }}
                      />
                      {label.name}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No labels</p>
              )}
            </SidebarSection>

            <SidebarSection title="Milestone">
              <div className="mb-3 flex items-center justify-between gap-2">
                <span className="t-xs">Track against a release.</span>
                {canEditMetadata ? (
                  <button
                    aria-expanded={openMetadataMenu === "milestone"}
                    className="btn sm"
                    disabled={isMutating}
                    onClick={() =>
                      setOpenMetadataMenu(
                        openMetadataMenu === "milestone" ? null : "milestone",
                      )
                    }
                    type="button"
                  >
                    Edit
                  </button>
                ) : null}
              </div>
              {openMetadataMenu === "milestone" ? (
                <MilestonePicker
                  disabled={isMutating}
                  milestones={pullRequest.metadataOptions.milestones}
                  onCancel={() => setOpenMetadataMenu(null)}
                  onSave={(milestone) => saveMetadata({ milestone })}
                  selectedMilestone={pullRequest.milestone}
                  title="Pull request milestone picker"
                />
              ) : null}
              {pullRequest.milestone ? (
                <span className="chip soft">{pullRequest.milestone.title}</span>
              ) : (
                <p className="t-xs">No milestone</p>
              )}
            </SidebarSection>

            <SidebarSection title="Linked issues">
              {pullRequest.linkedIssues.length ? (
                <div className="flex flex-col gap-2">
                  {pullRequest.linkedIssues.map((issue) => (
                    <Link
                      className="chip soft"
                      href={issue.href}
                      key={issue.number}
                    >
                      #{issue.number} · {issue.state}
                    </Link>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No linked issues</p>
              )}
            </SidebarSection>

            <SidebarSection title="Projects">
              <p className="t-xs">No project fields are connected.</p>
            </SidebarSection>

            <SidebarSection title="Notifications">
              <ThreadNotificationCard
                activePath={activePath}
                disabled={false}
                isMutating={isMutating}
                onSave={saveSubscription}
                subscription={subscription}
                viewerAuthenticated={viewerAuthenticated}
              />
            </SidebarSection>

            <SidebarSection title="Participants">
              {pullRequest.participants.length ? (
                <div className="flex flex-wrap gap-2">
                  {pullRequest.participants.slice(0, 12).map((participant) => (
                    <span
                      className="av sm"
                      key={participant.id}
                      title={participant.login}
                    >
                      {avatarLabel(participant.login)}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No participants yet</p>
              )}
            </SidebarSection>
          </aside>
        </div>
      </main>
    </RepositoryShell>
  );
}
