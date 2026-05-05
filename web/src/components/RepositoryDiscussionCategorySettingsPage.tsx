"use client";

import Link from "next/link";
import { type FormEvent, useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  DiscussionCategoryAdminItem,
  DiscussionCategoryFormat,
  DiscussionCategorySectionItem,
  DiscussionCategorySettingsView,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryDiscussionCategorySettingsPageProps = {
  repository: RepositoryOverview;
  settings: DiscussionCategorySettingsView | ApiErrorEnvelope;
};

type DialogMode =
  | { kind: "create" }
  | { category: DiscussionCategoryAdminItem; kind: "edit" };
type SectionDialogMode =
  | { kind: "create" }
  | { kind: "edit"; section: DiscussionCategorySectionItem };
type DeleteCategoryMode = { category: DiscussionCategoryAdminItem };

const formatOptions: Array<{ label: string; value: DiscussionCategoryFormat }> =
  [
    { label: "Question and Answer", value: "question_and_answer" },
    { label: "Open-ended", value: "open_ended" },
    { label: "Announcement", value: "announcement" },
    { label: "Poll", value: "poll" },
  ];

function isApiError(
  settings: DiscussionCategorySettingsView | ApiErrorEnvelope,
): settings is ApiErrorEnvelope {
  return "error" in settings;
}

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function formatLabel(format: DiscussionCategoryFormat) {
  return (
    formatOptions.find((option) => option.value === format)?.label ??
    String(format).replaceAll("_", " ")
  );
}

function categoryEndpoint(owner: string, repo: string, categoryId?: string) {
  const base = `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories`;
  return categoryId ? `${base}/${encodeURIComponent(categoryId)}` : base;
}

function sectionEndpoint(owner: string, repo: string, sectionId?: string) {
  const base = `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections`;
  return sectionId ? `${base}/${encodeURIComponent(sectionId)}` : base;
}

function categoryOrderEndpoint(owner: string, repo: string) {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/order`;
}

function sectionOrderEndpoint(owner: string, repo: string) {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections/order`;
}

function groupCategories(settings: DiscussionCategorySettingsView) {
  const sections = settings.sections.map((section) => ({
    id: section.id,
    name: section.name,
    position: section.position,
  }));
  return [
    { id: "unsectioned", name: "General categories", position: -1 },
    ...sections,
  ].map((section) => ({
    ...section,
    categories: settings.categories.filter((category) =>
      section.id === "unsectioned"
        ? category.sectionId === null
        : category.sectionId === section.id,
    ),
  }));
}

async function settingsMutation(
  endpoint: string,
  options: RequestInit,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(endpoint, options);
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion category settings could not be saved.",
    );
  }
  return payload as DiscussionCategorySettingsView;
}

function CategoryDialog({
  mode,
  onClose,
  onSaved,
  owner,
  repo,
  settings,
}: {
  mode: DialogMode;
  onClose: () => void;
  onSaved: (settings: DiscussionCategorySettingsView) => void;
  owner: string;
  repo: string;
  settings: DiscussionCategorySettingsView;
}) {
  const category = mode.kind === "edit" ? mode.category : null;
  const [emoji, setEmoji] = useState(category?.emoji ?? "💬");
  const [name, setName] = useState(category?.name ?? "");
  const [description, setDescription] = useState(category?.description ?? "");
  const [format, setFormat] = useState<DiscussionCategoryFormat>(
    category?.format ?? "question_and_answer",
  );
  const [sectionId, setSectionId] = useState(category?.sectionId ?? "");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      setError("Name is required.");
      return;
    }
    if (!emoji.trim()) {
      setError("Emoji is required.");
      return;
    }
    setPending(true);
    setError(null);

    const response = await fetch(categoryEndpoint(owner, repo, category?.id), {
      body: JSON.stringify({
        description: description.trim() || null,
        emoji: emoji.trim(),
        format,
        name: trimmedName,
        sectionId: sectionId || null,
      }),
      headers: { "content-type": "application/json" },
      method: category ? "PATCH" : "POST",
    });
    const payload = await response.json().catch(() => null);
    setPending(false);
    if (!response.ok) {
      const envelope = payload as ApiErrorEnvelope | null;
      setError(
        envelope?.error.message ?? "Discussion category could not be saved.",
      );
      return;
    }
    onSaved(payload as DiscussionCategorySettingsView);
    onClose();
  }

  return (
    <div
      aria-labelledby="discussion-category-dialog-title"
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center px-4"
      role="dialog"
      style={{
        background: "color-mix(in oklch, var(--ink-1) 24%, transparent)",
      }}
    >
      <form className="card w-full max-w-xl p-5" onSubmit={submit}>
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Discussion category
            </p>
            <h2 className="t-h2 mt-1" id="discussion-category-dialog-title">
              {category ? "Edit category" : "New category"}
            </h2>
          </div>
          <button className="btn ghost sm" onClick={onClose} type="button">
            Close
          </button>
        </div>

        {error ? (
          <div
            className="mt-4 rounded-[var(--radius)] border p-3 t-sm"
            style={{ background: "var(--err-soft)", borderColor: "var(--err)" }}
          >
            {error}
          </div>
        ) : null}

        <div className="mt-5 grid gap-4">
          <label className="grid gap-2">
            <span className="t-label">Emoji</span>
            <input
              aria-label="Category emoji"
              className="input"
              maxLength={16}
              onChange={(event) => setEmoji(event.target.value)}
              value={emoji}
            />
          </label>
          <label className="grid gap-2">
            <span className="t-label">Name</span>
            <input
              aria-label="Category name"
              className="input"
              maxLength={80}
              onChange={(event) => setName(event.target.value)}
              value={name}
            />
          </label>
          <label className="grid gap-2">
            <span className="t-label">Description</span>
            <textarea
              aria-label="Category description"
              className="input min-h-24"
              maxLength={280}
              onChange={(event) => setDescription(event.target.value)}
              value={description}
            />
          </label>
          <label className="grid gap-2">
            <span className="t-label">Format</span>
            <select
              aria-label="Category format"
              className="input"
              onChange={(event) =>
                setFormat(event.target.value as DiscussionCategoryFormat)
              }
              value={format}
            >
              {formatOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-2">
            <span className="t-label">Section</span>
            <select
              aria-label="Category section"
              className="input"
              onChange={(event) => setSectionId(event.target.value)}
              value={sectionId}
            >
              <option value="">General categories</option>
              {settings.sections.map((section) => (
                <option key={section.id} value={section.id}>
                  {section.name}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <button className="btn ghost" onClick={onClose} type="button">
            Cancel
          </button>
          <button className="btn primary" disabled={pending} type="submit">
            {pending
              ? "Saving..."
              : category
                ? "Save category"
                : "Create category"}
          </button>
        </div>
      </form>
    </div>
  );
}

function SectionDialog({
  mode,
  onClose,
  onSaved,
  owner,
  repo,
}: {
  mode: SectionDialogMode;
  onClose: () => void;
  onSaved: (settings: DiscussionCategorySettingsView) => void;
  owner: string;
  repo: string;
}) {
  const section = mode.kind === "edit" ? mode.section : null;
  const [name, setName] = useState(section?.name ?? "");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      setError("Section name is required.");
      return;
    }
    setPending(true);
    setError(null);
    try {
      const settings = await settingsMutation(
        sectionEndpoint(owner, repo, section?.id),
        {
          body: JSON.stringify({ name: trimmedName }),
          headers: { "content-type": "application/json" },
          method: section ? "PATCH" : "POST",
        },
      );
      onSaved(settings);
      onClose();
    } catch (error) {
      setError(
        error instanceof Error
          ? error.message
          : "Discussion category section could not be saved.",
      );
    } finally {
      setPending(false);
    }
  }

  return (
    <div
      aria-labelledby="discussion-section-dialog-title"
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center px-4"
      role="dialog"
      style={{
        background: "color-mix(in oklch, var(--ink-1) 24%, transparent)",
      }}
    >
      <form className="card w-full max-w-lg p-5" onSubmit={submit}>
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Discussion section
            </p>
            <h2 className="t-h2 mt-1" id="discussion-section-dialog-title">
              {section ? "Edit section" : "New section"}
            </h2>
          </div>
          <button className="btn ghost sm" onClick={onClose} type="button">
            Close
          </button>
        </div>
        {error ? (
          <div
            className="mt-4 rounded-[var(--radius)] border p-3 t-sm"
            style={{ background: "var(--err-soft)", borderColor: "var(--err)" }}
          >
            {error}
          </div>
        ) : null}
        <label className="mt-5 grid gap-2">
          <span className="t-label">Name</span>
          <input
            aria-label="Section name"
            className="input"
            maxLength={80}
            onChange={(event) => setName(event.target.value)}
            value={name}
          />
        </label>
        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <button className="btn ghost" onClick={onClose} type="button">
            Cancel
          </button>
          <button className="btn primary" disabled={pending} type="submit">
            {pending
              ? "Saving..."
              : section
                ? "Save section"
                : "Create section"}
          </button>
        </div>
      </form>
    </div>
  );
}

function DeleteCategoryDialog({
  mode,
  onClose,
  onSaved,
  owner,
  repo,
  settings,
}: {
  mode: DeleteCategoryMode;
  onClose: () => void;
  onSaved: (settings: DiscussionCategorySettingsView) => void;
  owner: string;
  repo: string;
  settings: DiscussionCategorySettingsView;
}) {
  const destinations = settings.categories.filter(
    (category) => category.id !== mode.category.id,
  );
  const [moveToCategoryId, setMoveToCategoryId] = useState(
    destinations[0]?.id ?? "",
  );
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setError(null);
    try {
      const settings = await settingsMutation(
        categoryEndpoint(owner, repo, mode.category.id),
        {
          body: JSON.stringify({ moveToCategoryId: moveToCategoryId || null }),
          headers: { "content-type": "application/json" },
          method: "DELETE",
        },
      );
      onSaved(settings);
      onClose();
    } catch (error) {
      setError(
        error instanceof Error
          ? error.message
          : "Discussion category could not be deleted.",
      );
    } finally {
      setPending(false);
    }
  }

  return (
    <div
      aria-labelledby="delete-category-dialog-title"
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center px-4"
      role="dialog"
      style={{
        background: "color-mix(in oklch, var(--ink-1) 24%, transparent)",
      }}
    >
      <form className="card w-full max-w-lg p-5" onSubmit={submit}>
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--err)" }}>
              Delete category
            </p>
            <h2 className="t-h2 mt-1" id="delete-category-dialog-title">
              Move discussions before deleting
            </h2>
          </div>
          <button className="btn ghost sm" onClick={onClose} type="button">
            Close
          </button>
        </div>
        <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
          Deleting {mode.category.name} moves its{" "}
          <span className="t-num">{formatNumber(mode.category.count)}</span>{" "}
          discussions to the destination category.
        </p>
        {error ? (
          <div
            className="mt-4 rounded-[var(--radius)] border p-3 t-sm"
            style={{ background: "var(--err-soft)", borderColor: "var(--err)" }}
          >
            {error}
          </div>
        ) : null}
        <label className="mt-5 grid gap-2">
          <span className="t-label">Destination category</span>
          <select
            aria-label="Destination category"
            className="input"
            onChange={(event) => setMoveToCategoryId(event.target.value)}
            value={moveToCategoryId}
          >
            {destinations.map((category) => (
              <option key={category.id} value={category.id}>
                {category.emoji} {category.name}
              </option>
            ))}
          </select>
        </label>
        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <button className="btn ghost" onClick={onClose} type="button">
            Cancel
          </button>
          <button
            className="btn"
            disabled={pending || !destinations.length}
            type="submit"
          >
            {pending ? "Deleting..." : "Delete and move"}
          </button>
        </div>
      </form>
    </div>
  );
}

function CategoryRow({
  category,
  canManage,
  onDelete,
  onEdit,
  onMove,
  sections,
}: {
  category: DiscussionCategoryAdminItem;
  canManage: boolean;
  onDelete: (category: DiscussionCategoryAdminItem) => void;
  onEdit: (category: DiscussionCategoryAdminItem) => void;
  onMove: (category: DiscussionCategoryAdminItem, sectionId: string) => void;
  sections: DiscussionCategorySectionItem[];
}) {
  return (
    <div className="list-row flex min-w-0 items-start gap-4 px-5 py-4">
      <span
        aria-hidden="true"
        className="grid h-11 w-11 shrink-0 place-items-center rounded-[var(--radius-lg)] text-2xl"
        style={{
          background: "var(--surface-2)",
          border: "1px solid var(--line-soft)",
        }}
      >
        {category.emoji}
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link
            className="break-words font-medium hover:underline"
            href={category.href}
          >
            {category.name}
          </Link>
          <span className={category.isPoll ? "chip warn" : "chip soft"}>
            {formatLabel(category.format)}
          </span>
          {category.acceptsAnswers ? (
            <span className="chip ok">Answers</span>
          ) : null}
          {category.isDefault ? (
            <span className="chip accent">Default</span>
          ) : null}
        </div>
        <p className="t-sm mt-1 break-words" style={{ color: "var(--ink-3)" }}>
          {category.description || "No description has been published."}
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          <span className="chip soft">
            <span className="t-num">{formatNumber(category.openCount)}</span>{" "}
            open
          </span>
          <span className="chip soft">
            <span className="t-num">{formatNumber(category.count)}</span> total
          </span>
          {category.templatePath ? (
            <Link className="chip soft" href={category.templateHref}>
              <span className="t-mono-sm">{category.templatePath}</span>
            </Link>
          ) : null}
        </div>
      </div>
      <div className="flex shrink-0 flex-wrap justify-end gap-2">
        <select
          aria-label={`Move ${category.name} to section`}
          className="input max-w-44"
          disabled={!canManage}
          onChange={(event) => onMove(category, event.target.value)}
          value={category.sectionId ?? ""}
        >
          <option value="">General categories</option>
          {sections.map((section) => (
            <option key={section.id} value={section.id}>
              {section.name}
            </option>
          ))}
        </select>
        <Link className="btn ghost sm" href={category.templateHref}>
          Template
        </Link>
        <button
          className="btn sm"
          disabled={!canManage}
          onClick={() => onEdit(category)}
          type="button"
        >
          Edit
        </button>
        <button
          className="btn ghost sm"
          disabled={!canManage}
          onClick={() => onMove(category, "__up__")}
          type="button"
        >
          Up
        </button>
        <button
          className="btn ghost sm"
          disabled={!canManage}
          onClick={() => onMove(category, "__down__")}
          type="button"
        >
          Down
        </button>
        <button
          className="btn ghost sm"
          disabled={!canManage}
          onClick={() => onDelete(category)}
          type="button"
        >
          Delete
        </button>
      </div>
    </div>
  );
}

function UnavailableSettings({
  repository,
  settings,
}: {
  repository: RepositoryOverview;
  settings: ApiErrorEnvelope;
}) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/discussions`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section className="card p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Category settings
        </p>
        <h1 className="t-h2 mt-1">Discussion categories are unavailable.</h1>
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          {settings.error.message}
        </p>
      </section>
    </RepositoryShell>
  );
}

export function RepositoryDiscussionCategorySettingsPage({
  repository,
  settings,
}: RepositoryDiscussionCategorySettingsPageProps) {
  const [current, setCurrent] = useState(settings);
  const [dialog, setDialog] = useState<DialogMode | null>(null);
  const [sectionDialog, setSectionDialog] = useState<SectionDialogMode | null>(
    null,
  );
  const [deleteCategory, setDeleteCategory] =
    useState<DeleteCategoryMode | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const owner = repository.owner_login;
  const repo = repository.name;
  const groups = useMemo(
    () => (isApiError(current) ? [] : groupCategories(current)),
    [current],
  );

  if (isApiError(current)) {
    return <UnavailableSettings repository={repository} settings={current} />;
  }

  const settingsView = current;
  const disabled = !current.enabled || !current.viewer.canManage;

  async function runAction(
    success: string,
    action: () => Promise<DiscussionCategorySettingsView>,
  ) {
    setPendingAction(success);
    setNotice(null);
    try {
      setCurrent(await action());
      setNotice(success);
    } catch (error) {
      setNotice(
        error instanceof Error
          ? error.message
          : "Discussion category settings could not be saved.",
      );
    } finally {
      setPendingAction(null);
    }
  }

  function orderedCategoryPayload() {
    return settingsView.categories.map((category, index) => ({
      id: category.id,
      position: index + 1,
      sectionId: category.sectionId,
    }));
  }

  function moveCategory(
    category: DiscussionCategoryAdminItem,
    sectionId: string,
  ) {
    if (sectionId === "__up__" || sectionId === "__down__") {
      moveCategoryPosition(category, sectionId === "__up__" ? -1 : 1);
      return;
    }
    void runAction("Category section assignment saved.", () =>
      settingsMutation(categoryOrderEndpoint(owner, repo), {
        body: JSON.stringify({
          items: orderedCategoryPayload().map((item) =>
            item.id === category.id
              ? { ...item, sectionId: sectionId || null }
              : item,
          ),
        }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      }),
    );
  }

  function moveCategoryPosition(
    category: DiscussionCategoryAdminItem,
    delta: -1 | 1,
  ) {
    const categories = [...settingsView.categories];
    const index = categories.findIndex((item) => item.id === category.id);
    const nextIndex = index + delta;
    if (index < 0 || nextIndex < 0 || nextIndex >= categories.length) return;
    const [selected] = categories.splice(index, 1);
    categories.splice(nextIndex, 0, selected);
    void runAction("Category order saved.", () =>
      settingsMutation(categoryOrderEndpoint(owner, repo), {
        body: JSON.stringify({
          items: categories.map((item, position) => ({
            id: item.id,
            position: position + 1,
            sectionId: item.sectionId,
          })),
        }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      }),
    );
  }

  function moveSectionPosition(
    section: DiscussionCategorySectionItem,
    delta: -1 | 1,
  ) {
    const sections = [...settingsView.sections];
    const index = sections.findIndex((item) => item.id === section.id);
    const nextIndex = index + delta;
    if (index < 0 || nextIndex < 0 || nextIndex >= sections.length) return;
    const [selected] = sections.splice(index, 1);
    sections.splice(nextIndex, 0, selected);
    void runAction("Section order saved.", () =>
      settingsMutation(sectionOrderEndpoint(owner, repo), {
        body: JSON.stringify({
          items: sections.map((item, position) => ({
            id: item.id,
            position: position + 1,
          })),
        }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      }),
    );
  }

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/discussions`}
      frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0 space-y-5">
        <section className="card p-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Repository settings
              </p>
              <h1 className="t-h2 mt-1">Discussion categories</h1>
              <p
                className="t-sm mt-2 max-w-3xl"
                style={{ color: "var(--ink-3)" }}
              >
                Manage the conversation formats maintainers offer to the
                community.
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <button
                className="btn"
                disabled={disabled}
                onClick={() => setSectionDialog({ kind: "create" })}
                type="button"
              >
                New section
              </button>
              <button
                className="btn primary"
                disabled={disabled || current.remainingCategories <= 0}
                onClick={() => setDialog({ kind: "create" })}
                type="button"
              >
                New category
              </button>
            </div>
          </div>
        </section>

        {!current.enabled ? (
          <section
            className="card p-4"
            style={{
              background: "var(--warn-soft)",
              borderColor: "var(--warn)",
            }}
          >
            <p className="t-label" style={{ color: "var(--warn)" }}>
              Discussions disabled
            </p>
            <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
              {current.disabledReason ??
                "Repository discussions are disabled by organization policy."}
            </p>
          </section>
        ) : null}

        {notice ? (
          <section className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Notice
            </p>
            <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
              {notice}
            </p>
            {pendingAction ? (
              <p className="t-xs mt-2">Saving {pendingAction}</p>
            ) : null}
          </section>
        ) : null}

        {groups.map((group) => (
          <section className="card overflow-hidden" key={group.id}>
            <div
              className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3"
              style={{
                background: "var(--surface-2)",
                borderColor: "var(--line)",
              }}
            >
              <div>
                <h2 className="t-h3">{group.name}</h2>
                <p className="t-xs mt-1">
                  <span className="t-num">{group.categories.length}</span>{" "}
                  categories
                </p>
              </div>
              {group.id === "unsectioned" ? null : (
                <div className="flex flex-wrap gap-2">
                  <button
                    className="btn ghost sm"
                    disabled={disabled}
                    onClick={() => {
                      const section = current.sections.find(
                        (item) => item.id === group.id,
                      );
                      if (section) setSectionDialog({ kind: "edit", section });
                    }}
                    type="button"
                  >
                    Edit section
                  </button>
                  <button
                    className="btn ghost sm"
                    disabled={disabled}
                    onClick={() => {
                      const section = current.sections.find(
                        (item) => item.id === group.id,
                      );
                      if (section) moveSectionPosition(section, -1);
                    }}
                    type="button"
                  >
                    Move up
                  </button>
                  <button
                    className="btn ghost sm"
                    disabled={disabled}
                    onClick={() => {
                      const section = current.sections.find(
                        (item) => item.id === group.id,
                      );
                      if (section) moveSectionPosition(section, 1);
                    }}
                    type="button"
                  >
                    Move down
                  </button>
                  <button
                    className="btn ghost sm"
                    disabled={disabled}
                    onClick={() => {
                      void runAction(
                        "Section deleted and categories moved.",
                        () =>
                          settingsMutation(
                            sectionEndpoint(owner, repo, group.id),
                            {
                              method: "DELETE",
                            },
                          ),
                      );
                    }}
                    type="button"
                  >
                    Delete section
                  </button>
                </div>
              )}
            </div>
            {group.categories.length ? (
              group.categories.map((category) => (
                <CategoryRow
                  canManage={!disabled}
                  category={category}
                  key={category.id}
                  onDelete={(selected) =>
                    setDeleteCategory({ category: selected })
                  }
                  onEdit={(selected) =>
                    setDialog({ category: selected, kind: "edit" })
                  }
                  onMove={moveCategory}
                  sections={current.sections}
                />
              ))
            ) : (
              <div className="p-5 t-sm" style={{ color: "var(--ink-3)" }}>
                No categories are assigned here yet.
              </div>
            )}
          </section>
        ))}
      </main>

      <aside className="space-y-4">
        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Admin state
          </p>
          <div className="mt-3 grid gap-2">
            <span
              className={current.viewer.canManage ? "chip ok" : "chip warn"}
            >
              {current.viewer.canManage ? "Can manage" : "Read only"}
            </span>
            <span className="chip soft">
              <span className="t-num">{current.remainingCategories}</span> of{" "}
              <span className="t-num">{current.categoryLimit}</span> slots left
            </span>
            <span className="chip soft capitalize">
              {current.viewer.permission ?? "signed out"}
            </span>
          </div>
        </section>

        <section className="card p-4">
          <h2 className="t-h3">Template forms</h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Category form templates are stored under{" "}
            <span className="t-mono-sm">.github/DISCUSSION_TEMPLATE</span>.
          </p>
        </section>
      </aside>

      {dialog ? (
        <CategoryDialog
          mode={dialog}
          onClose={() => setDialog(null)}
          onSaved={setCurrent}
          owner={owner}
          repo={repo}
          settings={current}
        />
      ) : null}
      {sectionDialog ? (
        <SectionDialog
          mode={sectionDialog}
          onClose={() => setSectionDialog(null)}
          onSaved={setCurrent}
          owner={owner}
          repo={repo}
        />
      ) : null}
      {deleteCategory ? (
        <DeleteCategoryDialog
          mode={deleteCategory}
          onClose={() => setDeleteCategory(null)}
          onSaved={setCurrent}
          owner={owner}
          repo={repo}
          settings={current}
        />
      ) : null}
    </RepositoryShell>
  );
}
