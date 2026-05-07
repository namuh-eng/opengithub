import Link from "next/link";
import { RepositoryCodeToolbar } from "@/components/RepositoryCodeToolbar";
import { RepositoryFileTable } from "@/components/RepositoryFileTable";
import { RepositoryQuickSetup } from "@/components/RepositoryQuickSetup";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  RepositoryAiSummary,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryCodeOverviewProps = {
  repository: RepositoryOverview;
  aiSummary?: RepositoryAiSummary | ApiErrorEnvelope | null;
};

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return Boolean(value && typeof value === "object" && "error" in value);
}

function formatCount(value: number, label: string) {
  return `${new Intl.NumberFormat("en").format(value)} ${label}`;
}

function RepositorySidebar({
  aiSummary,
  repository,
}: RepositoryCodeOverviewProps) {
  const base = `/${repository.owner_login}/${repository.name}`;

  return (
    <aside className="space-y-5 text-sm">
      <section>
        <h2 className="font-semibold" style={{ color: "var(--ink-1)" }}>
          About
        </h2>
        <p className="mt-3 leading-6" style={{ color: "var(--ink-1)" }}>
          {repository.sidebar.about ??
            "No description, website, or topics provided."}
        </p>
        {repository.sidebar.websiteUrl ? (
          <Link
            className="mt-2 block hover:underline"
            href={repository.sidebar.websiteUrl}
            style={{ color: "var(--accent)" }}
          >
            {repository.sidebar.websiteUrl}
          </Link>
        ) : null}
        {repository.sidebar.topics.length > 0 ? (
          <div className="mt-3 flex flex-wrap gap-2">
            {repository.sidebar.topics.map((topic) => (
              <Link
                className="chip accent"
                href={`/topics/${encodeURIComponent(topic)}`}
                key={topic}
              >
                {topic}
              </Link>
            ))}
          </div>
        ) : null}
      </section>
      <section
        className="card p-4"
        aria-label="AI repository summary"
        style={{ background: "var(--surface-2)" }}
      >
        <div className="flex items-center justify-between gap-3">
          <h2 className="t-label">AI summary</h2>
          <form
            action={`/${repository.owner_login}/${repository.name}/ai/summary`}
            method="post"
          >
            <button className="btn sm" type="submit">
              Regenerate
            </button>
          </form>
        </div>
        {aiSummary && !isApiError(aiSummary) && aiSummary.output ? (
          <p
            className="t-sm mt-3 whitespace-pre-wrap"
            style={{ color: "var(--ink-2)" }}
          >
            {aiSummary.output.output}
          </p>
        ) : aiSummary && !isApiError(aiSummary) && !aiSummary.enabled ? (
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            {aiSummary.reason}
          </p>
        ) : (
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            AI summary will appear after the first generation.
          </p>
        )}
      </section>
      <section className="space-y-2" style={{ color: "var(--ink-3)" }}>
        <Link
          className="block hover:underline"
          href={`${base}/stargazers`}
          style={{ color: "var(--ink-3)" }}
        >
          {formatCount(repository.sidebar.starsCount, "stars")}
        </Link>
        <p>{formatCount(repository.sidebar.watchersCount, "watching")}</p>
        <Link
          className="block hover:underline"
          href={`${base}/network/members`}
          style={{ color: "var(--ink-3)" }}
        >
          {formatCount(repository.sidebar.forksCount, "forks")}
        </Link>
        <Link
          className="block hover:underline"
          href={`${base}/releases`}
          style={{ color: "var(--ink-3)" }}
        >
          {formatCount(repository.sidebar.releasesCount, "releases")}
        </Link>
        <Link
          className="block hover:underline"
          href={`${base}/deployments`}
          style={{ color: "var(--ink-3)" }}
        >
          {formatCount(repository.sidebar.deploymentsCount, "deployments")}
        </Link>
        <p>
          {formatCount(repository.sidebar.contributorsCount, "contributors")}
        </p>
      </section>
      {repository.sidebar.languages.length > 0 ? (
        <section>
          <h2 className="font-semibold" style={{ color: "var(--ink-1)" }}>
            Languages
          </h2>
          <div
            className="mt-3 flex h-2 overflow-hidden rounded-full"
            style={{ background: "var(--line)" }}
          >
            {repository.sidebar.languages.map((language) => (
              <span
                aria-hidden="true"
                key={language.language}
                style={{
                  backgroundColor: language.color,
                  width: `${Math.max(language.percentage, 3)}%`,
                }}
              />
            ))}
          </div>
          <ul className="mt-3 flex flex-wrap gap-x-4 gap-y-2">
            {repository.sidebar.languages.map((language) => (
              <li className="flex items-center gap-1.5" key={language.language}>
                <span
                  aria-hidden="true"
                  className="h-3 w-3 rounded-full border border-black/10"
                  style={{ backgroundColor: language.color }}
                />
                <span>{language.language}</span>
                <span style={{ color: "var(--ink-3)" }}>
                  {language.percentage}%
                </span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </aside>
  );
}

export function RepositoryCodeOverview({
  aiSummary,
  repository,
}: RepositoryCodeOverviewProps) {
  return (
    <RepositoryShell repository={repository}>
      <div className="min-w-0 space-y-4">
        <RepositoryCodeToolbar repository={repository} />
        <RepositoryFileTable
          emptyState={<RepositoryQuickSetup repository={repository} />}
          entries={repository.rootEntries}
          historyHref={`/${repository.owner_login}/${repository.name}/commits/${repository.default_branch}`}
          latestCommit={repository.latestCommit}
        />
        {repository.readme ? (
          <article
            className="rounded-md"
            style={{
              border: "1px solid var(--line)",
              background: "var(--surface)",
            }}
          >
            <h2
              className="border-b px-4 py-3 t-sm font-semibold"
              style={{ borderColor: "var(--line)", color: "var(--ink-1)" }}
            >
              README.md
            </h2>
            <pre
              className="whitespace-pre-wrap px-4 py-4 t-sm leading-6"
              style={{ color: "var(--ink-1)" }}
            >
              {repository.readme.content}
            </pre>
          </article>
        ) : null}
      </div>
      <RepositorySidebar aiSummary={aiSummary} repository={repository} />
    </RepositoryShell>
  );
}
