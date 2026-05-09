"use client";

import { useEffect, useRef, useState, useTransition } from "react";
import type {
  RepositoryOverview,
  RepositorySocialState,
  RepositoryWatchEvent,
  RepositoryWatchLevel,
  RepositoryWatchSettings,
} from "@/lib/api";

type RepositoryHeaderActionsProps = {
  repository: RepositoryOverview;
};

function formatCompactCount(value: number) {
  return new Intl.NumberFormat("en", { notation: "compact" }).format(value);
}

function socialStateFromRepository(
  repository: RepositoryOverview,
): RepositorySocialState {
  return {
    starred: repository.viewerState.starred,
    watching: repository.viewerState.watching,
    watchLabel: repository.viewerState.watchLabel,
    watchLevel: repository.viewerState.watchLevel,
    customWatchEvents: repository.viewerState.customWatchEvents ?? [],
    forkedRepositoryHref: repository.viewerState.forkedRepositoryHref,
    starsCount: repository.sidebar.starsCount,
    watchersCount: repository.sidebar.watchersCount,
    forksCount: repository.sidebar.forksCount,
  };
}

async function mutateSocial(
  owner: string,
  repo: string,
  action: "star" | "watch",
  enabled: boolean,
) {
  const response = await fetch(`/${owner}/${repo}/actions/${action}`, {
    method: enabled ? "PUT" : "DELETE",
  });
  if (!response.ok) {
    throw new Error(`${action} update failed`);
  }
  return (await response.json()) as RepositorySocialState;
}

const watchOptions: Array<{
  level: RepositoryWatchLevel;
  label: string;
  shortLabel: string;
  accelerator: string;
  description: string;
}> = [
  {
    level: "participating",
    label: "Participating and @mentions",
    shortLabel: "Participating",
    accelerator: "P",
    description: "Notify when you participate, are assigned, or mentioned.",
  },
  {
    level: "all",
    label: "All Activity",
    shortLabel: "All Activity",
    accelerator: "A",
    description: "Notify for every conversation and repository event.",
  },
  {
    level: "ignore",
    label: "Ignore",
    shortLabel: "Ignoring",
    accelerator: "I",
    description: "Suppress repository notifications until you change this.",
  },
  {
    level: "custom",
    label: "Custom",
    shortLabel: "Custom",
    accelerator: "C",
    description: "Choose exactly which repository events notify you.",
  },
];

const fallbackWatchEvents: RepositoryWatchEvent[] = [
  "issues",
  "pull_requests",
  "releases",
  "discussions",
  "actions",
  "security_alerts",
  "repository_invitations",
];

const watchEventLabels: Record<RepositoryWatchEvent, string> = {
  issues: "Issue activity",
  pull_requests: "Pull request activity",
  releases: "Releases",
  discussions: "Discussions",
  actions: "Actions and CI",
  security_alerts: "Security alerts",
  repository_invitations: "Repository invitations",
};

function shortWatchLabel(social: RepositorySocialState) {
  if (!social.watching && social.watchLevel !== "ignore") {
    return "Watch";
  }
  return (
    watchOptions.find((option) => option.level === social.watchLevel)
      ?.shortLabel ??
    social.watchLabel ??
    "Watch"
  );
}

function socialFromWatchSettings(
  social: RepositorySocialState,
  settings: RepositoryWatchSettings,
): RepositorySocialState {
  return {
    ...social,
    watching: settings.watching,
    watchLabel: settings.label,
    watchLevel: settings.level,
    customWatchEvents: settings.customEvents,
    watchersCount: settings.watchersCount,
  };
}

export function RepositoryHeaderActions({
  repository,
}: RepositoryHeaderActionsProps) {
  const [social, setSocial] = useState(() =>
    socialStateFromRepository(repository),
  );
  const [feedback, setFeedback] = useState<string | null>(null);
  const [watchMenuOpen, setWatchMenuOpen] = useState(false);
  const [forkDialogOpen, setForkDialogOpen] = useState(false);
  const [forkDestinationOwner, setForkDestinationOwner] = useState(
    repository.viewerState.forkedRepositoryHref
      ? repository.viewerState.forkedRepositoryHref.split("/")[1] || ""
      : "",
  );
  const [forkName, setForkName] = useState(`${repository.name}-fork`);
  const [copyMainBranchOnly, setCopyMainBranchOnly] = useState(true);
  const [watchSettings, setWatchSettings] =
    useState<RepositoryWatchSettings | null>(null);
  const [selectedWatchLevel, setSelectedWatchLevel] =
    useState<RepositoryWatchLevel>(
      repository.viewerState.watchLevel ?? "participating",
    );
  const [selectedWatchEvents, setSelectedWatchEvents] = useState<
    RepositoryWatchEvent[]
  >(repository.viewerState.customWatchEvents ?? []);
  const [watchFeedback, setWatchFeedback] = useState<string | null>(null);
  const watchMenuRef = useRef<HTMLDivElement | null>(null);
  const watchDraftDirtyRef = useRef(false);
  const [isPending, startTransition] = useTransition();
  const owner = repository.owner_login;
  const repo = repository.name;
  const availableWatchEvents = watchSettings?.availableEvents.length
    ? watchSettings.availableEvents
    : fallbackWatchEvents;

  useEffect(() => {
    if (!watchMenuOpen) {
      return;
    }

    function onDocumentPointerDown(event: PointerEvent) {
      if (
        watchMenuRef.current &&
        !watchMenuRef.current.contains(event.target as Node)
      ) {
        setWatchMenuOpen(false);
      }
    }

    function onDocumentKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setWatchMenuOpen(false);
      }
    }

    document.addEventListener("pointerdown", onDocumentPointerDown);
    document.addEventListener("keydown", onDocumentKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onDocumentPointerDown);
      document.removeEventListener("keydown", onDocumentKeyDown);
    };
  }, [watchMenuOpen]);

  function setOptimisticSocial(
    next: RepositorySocialState,
    operation: () => Promise<RepositorySocialState>,
  ) {
    const previous = social;
    setSocial(next);
    setFeedback(null);
    startTransition(async () => {
      try {
        setSocial(await operation());
      } catch {
        setSocial(previous);
        setFeedback("Repository action could not be saved.");
      }
    });
  }

  function toggleStar() {
    const starred = !social.starred;
    setOptimisticSocial(
      {
        ...social,
        starred,
        starsCount: Math.max(0, social.starsCount + (starred ? 1 : -1)),
      },
      () => mutateSocial(owner, repo, "star", starred),
    );
  }

  function openWatchMenu() {
    const nextOpen = !watchMenuOpen;
    setWatchMenuOpen(nextOpen);
    if (!nextOpen) {
      return;
    }
    setWatchFeedback(null);
    watchDraftDirtyRef.current = false;
    startTransition(async () => {
      try {
        const response = await fetch(`/${owner}/${repo}/actions/watch`, {
          method: "GET",
        });
        if (!response.ok) {
          throw new Error("watch settings failed");
        }
        const settings = (await response.json()) as RepositoryWatchSettings;
        setWatchSettings(settings);
        if (!watchDraftDirtyRef.current) {
          setSelectedWatchLevel(settings.level);
          setSelectedWatchEvents(settings.customEvents);
        }
        setSocial((current) => socialFromWatchSettings(current, settings));
      } catch {
        setWatchFeedback("Watch settings could not be loaded.");
      }
    });
  }

  function toggleWatchEvent(event: RepositoryWatchEvent) {
    watchDraftDirtyRef.current = true;
    setSelectedWatchEvents((current) =>
      current.includes(event)
        ? current.filter((item) => item !== event)
        : [...current, event],
    );
  }

  function selectWatchLevel(level: RepositoryWatchLevel) {
    watchDraftDirtyRef.current = true;
    setSelectedWatchLevel(level);
  }

  function saveWatchSettings() {
    if (selectedWatchLevel === "custom" && selectedWatchEvents.length === 0) {
      setWatchFeedback("Choose at least one custom event.");
      return;
    }
    setWatchFeedback(null);
    startTransition(async () => {
      try {
        const response = await fetch(`/${owner}/${repo}/actions/watch`, {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            level: selectedWatchLevel,
            customEvents:
              selectedWatchLevel === "custom" ? selectedWatchEvents : [],
          }),
        });
        if (!response.ok) {
          throw new Error("watch update failed");
        }
        const settings = (await response.json()) as RepositoryWatchSettings;
        setWatchSettings(settings);
        setSelectedWatchLevel(settings.level);
        setSelectedWatchEvents(settings.customEvents);
        setSocial((current) => socialFromWatchSettings(current, settings));
        setWatchFeedback("Watch settings saved.");
        setWatchMenuOpen(false);
      } catch {
        setWatchFeedback("Watch settings could not be saved.");
      }
    });
  }

  function forkRepository() {
    const destinationOwner = forkDestinationOwner.trim();
    const name = forkName.trim();
    if (!name) {
      setFeedback("Fork repository name is required.");
      return;
    }
    setFeedback(null);
    startTransition(async () => {
      try {
        const response = await fetch(`/${owner}/${repo}/actions/fork`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            destinationOwner: destinationOwner || undefined,
            name,
            mainBranchOnly: copyMainBranchOnly,
          }),
        });
        if (!response.ok) {
          throw new Error("fork failed");
        }
        const result = (await response.json()) as {
          forkHref: string;
          social: RepositorySocialState;
        };
        setSocial(result.social);
        setForkDialogOpen(false);
        window.location.assign(result.forkHref);
      } catch {
        setFeedback("Repository could not be forked.");
      }
    });
  }

  return (
    <div className="flex flex-wrap items-center justify-end gap-2 text-sm">
      <div className="relative" ref={watchMenuRef}>
        <button
          aria-expanded={watchMenuOpen}
          aria-haspopup="menu"
          aria-pressed={social.watching}
          className="btn sm disabled:opacity-60"
          disabled={isPending && !watchMenuOpen}
          onClick={openWatchMenu}
          type="button"
        >
          <span>{shortWatchLabel(social)}</span>
          <span className="chip soft ml-1" style={{ marginLeft: "0.25rem" }}>
            {formatCompactCount(social.watchersCount)}
          </span>
        </button>
        {watchMenuOpen ? (
          <div
            aria-label="Repository watch settings"
            className="card absolute right-0 z-20 mt-2 w-[min(92vw,360px)] p-3 text-left shadow-lg"
            role="menu"
            style={{ background: "var(--surface)", color: "var(--ink-1)" }}
          >
            <div
              className="flex items-start justify-between gap-3 border-b pb-3"
              style={{ borderColor: "var(--line)" }}
            >
              <div>
                <p className="t-label">Notifications</p>
                <p className="t-xs mt-1">
                  {formatCompactCount(social.watchersCount)} watching this
                  repository
                </p>
              </div>
              <button
                aria-label="Close watch settings"
                className="btn ghost sm"
                onClick={() => setWatchMenuOpen(false)}
                type="button"
              >
                x
              </button>
            </div>
            <div className="mt-3 space-y-2" role="radiogroup">
              {watchOptions.map((option) => (
                <label
                  className="flex cursor-pointer items-start gap-3 rounded-md border p-3"
                  key={option.level}
                  style={{
                    borderColor:
                      selectedWatchLevel === option.level
                        ? "var(--accent)"
                        : "var(--line)",
                    background:
                      selectedWatchLevel === option.level
                        ? "var(--accent-soft)"
                        : "var(--surface)",
                  }}
                >
                  <input
                    checked={selectedWatchLevel === option.level}
                    className="mt-1"
                    name="repository-watch-level"
                    onChange={() => selectWatchLevel(option.level)}
                    type="radio"
                    value={option.level}
                  />
                  <span className="min-w-0 flex-1">
                    <span className="flex items-center justify-between gap-2">
                      <span className="t-sm font-semibold">{option.label}</span>
                      <span className="kbd">{option.accelerator}</span>
                    </span>
                    <span className="t-xs mt-1 block">
                      {option.description}
                    </span>
                  </span>
                </label>
              ))}
            </div>
            {selectedWatchLevel === "ignore" ? (
              <p className="chip warn mt-3 block whitespace-normal">
                {watchSettings?.ignoreWarning ??
                  "Ignoring this repository suppresses repository watch notifications until you choose another watch level."}
              </p>
            ) : null}
            {selectedWatchLevel === "custom" ? (
              <fieldset className="mt-3 space-y-2">
                <legend className="t-label">Event Types</legend>
                {availableWatchEvents.map((event) => (
                  <label
                    className="flex cursor-pointer items-center gap-2 t-sm"
                    key={event}
                  >
                    <input
                      checked={selectedWatchEvents.includes(event)}
                      onChange={() => toggleWatchEvent(event)}
                      type="checkbox"
                    />
                    <span>{watchEventLabels[event]}</span>
                  </label>
                ))}
              </fieldset>
            ) : null}
            {watchFeedback ? (
              <p
                className="mt-3 t-xs"
                role="status"
                style={{
                  color: watchFeedback.includes("saved")
                    ? "var(--ok)"
                    : "var(--err)",
                }}
              >
                {watchFeedback}
              </p>
            ) : null}
            <div className="mt-4 flex justify-end gap-2">
              <button
                className="btn ghost sm"
                onClick={() => setWatchMenuOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn accent sm disabled:opacity-60"
                disabled={isPending}
                onClick={saveWatchSettings}
                type="button"
              >
                Save
              </button>
            </div>
          </div>
        ) : null}
      </div>
      {social.forkedRepositoryHref ? (
        <a className="btn sm" href={social.forkedRepositoryHref}>
          Forked
          <span className="chip soft" style={{ marginLeft: "0.25rem" }}>
            {formatCompactCount(social.forksCount)}
          </span>
        </a>
      ) : (
        <button
          className="btn sm disabled:opacity-60"
          disabled={isPending}
          onClick={() => {
            setFeedback(null);
            setForkDialogOpen(true);
          }}
          type="button"
        >
          Fork
          <span className="chip soft" style={{ marginLeft: "0.25rem" }}>
            {formatCompactCount(social.forksCount)}
          </span>
        </button>
      )}
      {forkDialogOpen ? (
        <div
          aria-labelledby="fork-repository-title"
          aria-modal="true"
          className="fixed inset-0 z-40 flex items-center justify-center bg-black/20 p-4"
          role="dialog"
        >
          <div
            className="card w-[min(94vw,460px)] p-5 text-left shadow-lg"
            style={{ background: "var(--surface)", color: "var(--ink-1)" }}
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="t-label">Fork Repository</p>
                <h2 className="t-h2 mt-1" id="fork-repository-title">
                  Create a new fork
                </h2>
              </div>
              <button
                aria-label="Close fork dialog"
                className="btn ghost sm"
                onClick={() => setForkDialogOpen(false)}
                type="button"
              >
                x
              </button>
            </div>
            <div className="mt-4 space-y-3">
              <label className="block">
                <span className="t-label mb-1 block">Destination Owner</span>
                <input
                  className="input w-full"
                  onChange={(event) =>
                    setForkDestinationOwner(event.currentTarget.value)
                  }
                  placeholder="Your account"
                  value={forkDestinationOwner}
                />
              </label>
              <label className="block">
                <span className="t-label mb-1 block">Repository Name</span>
                <input
                  className="input w-full"
                  onChange={(event) => setForkName(event.currentTarget.value)}
                  value={forkName}
                />
              </label>
              <label className="flex items-center gap-2 t-sm">
                <input
                  checked={copyMainBranchOnly}
                  onChange={(event) =>
                    setCopyMainBranchOnly(event.currentTarget.checked)
                  }
                  type="checkbox"
                />
                <span>Copy main branch only</span>
              </label>
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <button
                className="btn ghost sm"
                onClick={() => setForkDialogOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn accent sm disabled:opacity-60"
                disabled={isPending}
                onClick={forkRepository}
                type="button"
              >
                Create fork
              </button>
            </div>
          </div>
        </div>
      ) : null}
      <button
        aria-pressed={social.starred}
        className="btn sm disabled:opacity-60"
        disabled={isPending}
        onClick={toggleStar}
        type="button"
      >
        {social.starred ? "Unstar" : "Star"}
        <span className="chip soft" style={{ marginLeft: "0.25rem" }}>
          {formatCompactCount(social.starsCount)}
        </span>
      </button>
      {feedback ? (
        <p
          className="basis-full text-right text-xs"
          role="alert"
          style={{ color: "var(--err)" }}
        >
          {feedback}
        </p>
      ) : null}
    </div>
  );
}
