"use client";

import Link from "next/link";
import type { ReactNode } from "react";
import { useState } from "react";
import { IssueTimeline, ReactionToolbar } from "@/components/IssueTimeline";
import { LabelPicker } from "@/components/LabelPicker";
import { MarkdownBody } from "@/components/MarkdownBody";
import { MilestonePicker } from "@/components/MilestonePicker";
import { RepositoryShell } from "@/components/RepositoryShell";
import { ThreadNotificationCard } from "@/components/ThreadNotificationCard";
import type {
  ApiErrorEnvelope,
  ConvertIssueToDiscussionResponse,
  IssueDetailView,
  IssueDiscussionConversionView,
  IssueListLabel,
  IssueListMilestone,
  IssueListUser,
  IssueTimelineItem,
  ReactionContent,
  RepositoryOverview,
  ThreadSubscriptionEvent,
} from "@/lib/api";
import {
  repositoryIssueDetailHref,
  repositoryIssuesHref,
} from "@/lib/navigation";

type RepositoryIssueDetailPageProps = {
  repository: RepositoryOverview;
  issue: IssueDetailView;
  timeline: IssueTimelineItem[];
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
  if (months < 12) {
    return `${months}mo ago`;
  }
  return `${Math.floor(months / 12)}y ago`;
}

function bytesLabel(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  const kib = value / 1024;
  if (kib < 1024) {
    return `${kib.toFixed(1)} KB`;
  }
  return `${(kib / 1024).toFixed(1)} MB`;
}

function avatarLabel(login: string) {
  return login.slice(0, 1).toUpperCase();
}

function SidebarSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="border-b py-4" style={{ borderColor: "var(--line)" }}>
      <h2 className="t-label mb-3">{title}</h2>
      {children}
    </section>
  );
}

function optionSelected(id: string, selectedIds: string[]) {
  return selectedIds.includes(id);
}

export function RepositoryIssueDetailPage({
  repository,
  issue,
  timeline,
  viewerAuthenticated,
}: RepositoryIssueDetailPageProps) {
  const issueHref = repositoryIssueDetailHref(
    repository.owner_login,
    repository.name,
    issue.number,
  );
  const issueListHref = repositoryIssuesHref(
    repository.owner_login,
    repository.name,
    { state: issue.state },
  );
  const bodyLabelId = `issue-${issue.number}-body`;
  const [currentIssue, setCurrentIssue] = useState(issue);
  const [reactions, setReactions] = useState(issue.reactions);
  const [subscription, setSubscription] = useState(issue.subscription);
  const [message, setMessage] = useState<string | null>(null);
  const [isMutating, setIsMutating] = useState(false);
  const [conversion, setConversion] =
    useState<IssueDiscussionConversionView | null>(null);
  const [conversionOpen, setConversionOpen] = useState(false);
  const [selectedCategorySlug, setSelectedCategorySlug] = useState("");
  const [openMetadataMenu, setOpenMetadataMenu] = useState<
    "assignees" | "labels" | "milestone" | null
  >(null);
  const stateOpen = currentIssue.state === "open";
  const canEditMetadata =
    viewerAuthenticated && currentIssue.viewerPermission !== null;

  async function saveMetadata(next: {
    labels?: IssueListLabel[];
    assignees?: IssueListUser[];
    milestone?: IssueListMilestone | null;
  }) {
    setMessage(null);
    setIsMutating(true);
    try {
      const nextLabels = next.labels ?? currentIssue.labels;
      const nextAssignees = next.assignees ?? currentIssue.assignees;
      const nextMilestone =
        "milestone" in next ? next.milestone : currentIssue.milestone;
      const response = await fetch(`${issueHref}/metadata`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          labelIds: nextLabels.map((label) => label.id),
          assigneeUserIds: nextAssignees.map((assignee) => assignee.id),
          milestoneId: nextMilestone?.id ?? null,
        }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Issue metadata could not be updated.",
        );
      }
      setCurrentIssue(payload as IssueDetailView);
      setOpenMetadataMenu(null);
      setMessage("Issue metadata updated.");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Issue metadata could not be updated.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  function saveLabels(labels: IssueListLabel[]) {
    void saveMetadata({ labels });
  }

  function toggleAssignee(assignee: IssueListUser) {
    const selectedIds = currentIssue.assignees.map((item) => item.id);
    const assignees = optionSelected(assignee.id, selectedIds)
      ? currentIssue.assignees.filter((item) => item.id !== assignee.id)
      : [...currentIssue.assignees, assignee];
    void saveMetadata({ assignees });
  }

  async function updateState(nextState: "open" | "closed") {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${issueHref}/state`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ state: nextState }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Issue state could not be updated.",
        );
      }
      setCurrentIssue(payload as IssueDetailView);
      setMessage(nextState === "closed" ? "Issue closed." : "Issue reopened.");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Issue state could not be updated.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  async function toggleReaction(content: ReactionContent) {
    setMessage(null);
    try {
      const response = await fetch(`${issueHref}/reactions`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ content }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Reaction could not be updated.",
        );
      }
      setReactions(payload);
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Reaction could not be updated.",
      );
    }
  }

  async function saveSubscription(
    subscribed: boolean,
    customEvents: ThreadSubscriptionEvent[],
  ) {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${issueHref}/subscription`, {
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
      setSubscription(payload);
      setMessage(
        payload.subscribed ? "Subscribed to notifications." : "Unsubscribed.",
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

  async function openConversionDialog() {
    setMessage(null);
    setConversionOpen(true);
    if (conversion) {
      return;
    }
    setIsMutating(true);
    try {
      const response = await fetch(`${issueHref}/convert-to-discussion`);
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ??
            "Discussion conversion metadata could not be loaded.",
        );
      }
      const view = payload as IssueDiscussionConversionView;
      setConversion(view);
      setSelectedCategorySlug(
        view.categories.find((category) => !category.disabledReason)?.slug ??
          "",
      );
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Discussion conversion metadata could not be loaded.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  async function convertToDiscussion() {
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(`${issueHref}/convert-to-discussion`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ categorySlug: selectedCategorySlug }),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = payload as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ??
            "Issue could not be converted to a discussion.",
        );
      }
      const converted = payload as ConvertIssueToDiscussionResponse;
      setMessage(`Converted to discussion #${converted.discussionNumber}.`);
      window.location.assign(converted.href);
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Issue could not be converted to a discussion.",
      );
    } finally {
      setIsMutating(false);
    }
  }

  return (
    <RepositoryShell
      activePath={issueHref}
      frameClassName="max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0">
        <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <Link className="btn sm" href={issueListHref}>
                All issues
              </Link>
              <Link
                className="btn primary sm"
                href={`/${repository.owner_login}/${repository.name}/issues/new`}
              >
                New issue
              </Link>
            </div>
            <h1 className="t-h1 break-words">
              {currentIssue.title}{" "}
              <span className="t-num" style={{ color: "var(--ink-4)" }}>
                #{currentIssue.number}
              </span>
            </h1>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <span className={`chip ${stateOpen ? "ok" : "soft"}`}>
                {stateOpen ? "Open" : "Closed"}
              </span>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                <strong style={{ color: "var(--ink-1)" }}>
                  {currentIssue.author.login}
                </strong>{" "}
                opened this issue {relativeTime(currentIssue.createdAt)} ·{" "}
                <span className="t-num">{currentIssue.commentCount}</span>{" "}
                {currentIssue.commentCount === 1 ? "comment" : "comments"}
              </p>
            </div>
            {message ? (
              <p
                className="mt-3 t-sm"
                role={message.includes("could not") ? "alert" : "status"}
                style={{
                  color: message.includes("could not")
                    ? "var(--err)"
                    : "var(--ok)",
                }}
              >
                {message}
              </p>
            ) : null}
          </div>
          {viewerAuthenticated ? (
            <button
              className="btn"
              disabled={isMutating}
              onClick={() => void updateState(stateOpen ? "closed" : "open")}
              type="button"
            >
              {isMutating
                ? "Updating..."
                : stateOpen
                  ? "Close issue"
                  : "Reopen issue"}
            </button>
          ) : null}
        </div>

        <div className="grid grid-cols-[minmax(0,1fr)_296px] gap-8 max-lg:grid-cols-1">
          <div className="min-w-0">
            <article className="flex gap-4">
              <div className="av lg shrink-0" aria-hidden="true">
                {avatarLabel(currentIssue.author.login)}
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
                    {currentIssue.author.login}
                  </h2>
                  <span className="t-xs">
                    opened {relativeTime(currentIssue.createdAt)}
                  </span>
                  <span className="chip soft ml-auto">author</span>
                </div>
                <div className="p-5">
                  {currentIssue.body?.trim() ? (
                    <MarkdownBody
                      html={currentIssue.bodyHtml}
                      labelledBy={bodyLabelId}
                    />
                  ) : (
                    <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                      No description provided.
                    </p>
                  )}
                  {viewerAuthenticated ? (
                    <ReactionToolbar
                      label="Issue reactions"
                      onToggle={(content) => void toggleReaction(content)}
                      reactions={reactions}
                    />
                  ) : null}
                </div>
              </div>
            </article>

            <div className="mt-6">
              <IssueTimeline
                initialItems={timeline}
                issueState={currentIssue.state}
                issueNumber={issue.number}
                loginHref={`/login?next=${encodeURIComponent(issueHref)}`}
                onStateChanged={(state) =>
                  setCurrentIssue((current) => ({ ...current, state }))
                }
                owner={repository.owner_login}
                repo={repository.name}
                viewerAuthenticated={viewerAuthenticated}
              />
            </div>
          </div>

          <aside className="min-w-0">
            <SidebarSection title="Assignees">
              <div className="mb-3 flex items-center justify-between gap-2">
                <span className="t-xs">People responsible for this issue.</span>
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
                  {currentIssue.metadataOptions.assignees.length ? (
                    currentIssue.metadataOptions.assignees.map((assignee) => {
                      const selected = currentIssue.assignees.some(
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
              {currentIssue.assignees.length ? (
                <div className="flex flex-col gap-2">
                  {currentIssue.assignees.map((assignee) => (
                    <div className="row gap-2" key={assignee.id}>
                      <div className="av sm" aria-hidden="true">
                        {avatarLabel(assignee.login)}
                      </div>
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
                <span className="t-xs">Classify and triage this issue.</span>
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
                  labels={currentIssue.metadataOptions.labels}
                  onCancel={() => setOpenMetadataMenu(null)}
                  onSave={saveLabels}
                  selectedLabels={currentIssue.labels}
                  title="Issue label picker"
                />
              ) : null}
              {currentIssue.labels.length ? (
                <div className="flex flex-wrap gap-2">
                  {currentIssue.labels.map((label) => (
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
                <span className="t-xs">
                  Track this issue against a release.
                </span>
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
                  milestones={currentIssue.metadataOptions.milestones}
                  onCancel={() => setOpenMetadataMenu(null)}
                  onSave={(milestone) => void saveMetadata({ milestone })}
                  selectedMilestone={currentIssue.milestone}
                  title="Issue milestone picker"
                />
              ) : null}
              {currentIssue.milestone ? (
                <span className="chip soft">
                  {currentIssue.milestone.title}
                </span>
              ) : (
                <p className="t-xs">No milestone</p>
              )}
            </SidebarSection>

            <SidebarSection title="Type">
              <p className="t-xs">Issue types are not configured.</p>
            </SidebarSection>

            <SidebarSection title="Fields">
              <p className="t-xs">No custom issue fields are configured.</p>
            </SidebarSection>

            <SidebarSection title="Projects">
              <p className="t-xs">No project fields are connected.</p>
            </SidebarSection>

            <SidebarSection title="Development">
              {currentIssue.linkedPullRequest ? (
                <Link
                  className="chip soft"
                  href={currentIssue.linkedPullRequest.href}
                >
                  PR #{currentIssue.linkedPullRequest.number} ·{" "}
                  {currentIssue.linkedPullRequest.state}
                </Link>
              ) : (
                <p className="t-xs">No linked pull requests</p>
              )}
            </SidebarSection>

            <SidebarSection title="Convert">
              <p className="t-xs mb-3">
                Move this issue into Discussions while preserving the source
                issue timeline link.
              </p>
              {viewerAuthenticated && canEditMetadata ? (
                <button
                  className="btn sm w-full justify-center"
                  disabled={isMutating}
                  onClick={() => void openConversionDialog()}
                  type="button"
                >
                  Convert to discussion
                </button>
              ) : (
                <p className="t-xs">
                  Maintainer access is required to convert issues.
                </p>
              )}
              {conversionOpen ? (
                <div className="card mt-3 p-3" role="dialog">
                  <div className="mb-3 flex items-start justify-between gap-3">
                    <div>
                      <h3 className="t-h3">Convert issue</h3>
                      <p className="t-xs mt-1">
                        The issue will close and link to the new discussion.
                        {conversion
                          ? ` ${conversion.commentCount} issue comments will be copied as discussion comments.`
                          : ""}
                      </p>
                    </div>
                    <button
                      className="btn ghost sm"
                      onClick={() => setConversionOpen(false)}
                      type="button"
                    >
                      Close
                    </button>
                  </div>
                  {conversion?.alreadyConverted &&
                  conversion.convertedDiscussionHref ? (
                    <Link
                      className="chip soft"
                      href={conversion.convertedDiscussionHref}
                    >
                      Already converted to discussion #
                      {conversion.convertedDiscussionNumber}
                    </Link>
                  ) : (
                    <div className="flex flex-col gap-3">
                      <label className="t-label" htmlFor="convert-category">
                        Discussion category
                      </label>
                      <select
                        className="input"
                        disabled={isMutating || !conversion}
                        id="convert-category"
                        onChange={(event) =>
                          setSelectedCategorySlug(event.currentTarget.value)
                        }
                        value={selectedCategorySlug}
                      >
                        {conversion?.categories.map((category) => (
                          <option
                            disabled={Boolean(category.disabledReason)}
                            key={category.id}
                            value={category.slug}
                          >
                            {category.emoji} {category.name}
                            {category.disabledReason
                              ? ` - ${category.disabledReason}`
                              : ""}
                          </option>
                        ))}
                      </select>
                      {conversion?.disabledReason ? (
                        <p className="t-xs" style={{ color: "var(--err)" }}>
                          {conversion.disabledReason}
                        </p>
                      ) : null}
                      <button
                        className="btn primary sm"
                        disabled={
                          isMutating ||
                          !conversion?.canConvert ||
                          selectedCategorySlug.length === 0
                        }
                        onClick={() => void convertToDiscussion()}
                        type="button"
                      >
                        {isMutating ? "Converting..." : "Convert issue"}
                      </button>
                    </div>
                  )}
                </div>
              ) : null}
            </SidebarSection>

            <SidebarSection title="Attachments">
              {currentIssue.attachments.length ? (
                <div className="flex flex-col gap-2">
                  {currentIssue.attachments.map((attachment) => (
                    <div className="card p-3" key={attachment.id}>
                      <p className="t-sm font-medium">{attachment.fileName}</p>
                      <p className="t-xs">
                        {bytesLabel(attachment.byteSize)} ·{" "}
                        {attachment.storageStatus.replaceAll("_", " ")}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="t-xs">No attachments</p>
              )}
            </SidebarSection>

            <SidebarSection title="Notifications">
              <ThreadNotificationCard
                activePath={issueHref}
                disabled={false}
                events={["closed", "reopened"]}
                isMutating={isMutating}
                onSave={saveSubscription}
                subscription={subscription}
                viewerAuthenticated={viewerAuthenticated}
              />
            </SidebarSection>

            <SidebarSection title="Participants">
              {currentIssue.participants.length ? (
                <div className="flex flex-wrap gap-1">
                  {currentIssue.participants.map((participant) => (
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
