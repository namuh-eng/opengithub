"use client";

import Link from "next/link";
import type { ReactNode } from "react";
import { useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { PullRequestTimeline } from "@/components/PullRequestTimeline";
import { RepositoryShell } from "@/components/RepositoryShell";
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
  const pullRequest = currentPullRequest;
  const branchProtection = pullRequest.mergeability.branchProtection ?? {
    protected: false,
    pattern: null,
    requiredApprovingReviewCount: 0,
    requiresUpToDateBranch: false,
    requiredStatusChecks: [],
  };
  const bodyLabelId = `pull-request-${pullRequest.number}-body`;
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const activePath = `${basePath}/pull/${pullRequest.number}`;
  const canEditMetadata =
    viewerAuthenticated && currentPullRequest.viewerPermission !== null;
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
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/merge`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ method: mergeMethod }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Pull request could not merge.",
        );
      }
      const updated = payload as PullRequestDetailView;
      setCurrentPullRequest(updated);
      setSubscription(updated.subscription);
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

  function toggleLabel(label: IssueListLabel) {
    const selectedIds = currentPullRequest.labels.map((item) => item.id);
    const labels = optionSelected(label.id, selectedIds)
      ? currentPullRequest.labels.filter((item) => item.id !== label.id)
      : [...currentPullRequest.labels, label];
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

  async function toggleSubscription() {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${activePath}/subscription`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ subscribed: !subscription.subscribed }),
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
                      onClick={() => setMergeMethod(method)}
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
                    className="btn accent"
                    disabled={isMutating || !pullRequest.mergeability.canMerge}
                    onClick={() => void mergePullRequest()}
                    type="button"
                  >
                    Merge pull request
                  </button>
                </div>
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
                <div className="card mb-3 p-2" role="menu">
                  {pullRequest.metadataOptions.labels.length ? (
                    pullRequest.metadataOptions.labels.map((label) => {
                      const selected = pullRequest.labels.some(
                        (item) => item.id === label.id,
                      );
                      return (
                        <button
                          aria-pressed={selected}
                          className="btn ghost sm w-full justify-start"
                          key={label.id}
                          onClick={() => toggleLabel(label)}
                          type="button"
                        >
                          <span
                            aria-hidden="true"
                            className="inline-block h-2 w-2 rounded-full"
                            style={{ background: label.color }}
                          />
                          {selected ? "Remove" : "Add"} {label.name}
                        </button>
                      );
                    })
                  ) : (
                    <p className="t-xs p-2">No labels configured.</p>
                  )}
                </div>
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
                <div className="card mb-3 p-2" role="menu">
                  <button
                    aria-pressed={pullRequest.milestone === null}
                    className="btn ghost sm w-full justify-start"
                    onClick={() => saveMetadata({ milestone: null })}
                    type="button"
                  >
                    No milestone
                  </button>
                  {pullRequest.metadataOptions.milestones.map((milestone) => (
                    <button
                      aria-pressed={pullRequest.milestone?.id === milestone.id}
                      className="btn ghost sm w-full justify-start"
                      key={milestone.id}
                      onClick={() => saveMetadata({ milestone })}
                      type="button"
                    >
                      {milestone.title}
                    </button>
                  ))}
                </div>
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
              <p className="t-xs">
                {subscription.subscribed
                  ? `Subscribed: ${subscription.reason}`
                  : "Not subscribed"}
              </p>
              {viewerAuthenticated ? (
                <button
                  className="btn sm mt-3"
                  disabled={isMutating}
                  onClick={() => void toggleSubscription()}
                  type="button"
                >
                  {subscription.subscribed ? "Unsubscribe" : "Subscribe"}
                </button>
              ) : (
                <Link
                  className="btn sm mt-3"
                  href={`/login?next=${encodeURIComponent(activePath)}`}
                >
                  Sign in to subscribe
                </Link>
              )}
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
