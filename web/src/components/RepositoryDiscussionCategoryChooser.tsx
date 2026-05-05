import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  DiscussionCategoryChoice,
  DiscussionCreationView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryNewDiscussionHref } from "@/lib/navigation";

type RepositoryDiscussionCategoryChooserProps = {
  creation: DiscussionCreationView;
  repository: RepositoryOverview;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function categoryKind(category: DiscussionCategoryChoice) {
  if (category.isPoll) return "Poll";
  if (category.acceptsAnswers) return "Answers";
  return "Open-ended";
}

function CategoryCard({
  category,
  owner,
  repo,
  disabled,
}: {
  category: DiscussionCategoryChoice;
  owner: string;
  repo: string;
  disabled: boolean;
}) {
  const href =
    category.formHref ||
    repositoryNewDiscussionHref(owner, repo, { category: category.slug });

  return (
    <article className="card flex min-w-0 flex-col p-5">
      <div className="flex min-w-0 items-start gap-4">
        <span
          aria-hidden="true"
          className="grid h-12 w-12 shrink-0 place-items-center rounded-[var(--radius-lg)] text-2xl"
          style={{
            background: "var(--surface-2)",
            border: "1px solid var(--line-soft)",
          }}
        >
          {category.emoji}
        </span>
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h2 className="break-words t-h3">{category.name}</h2>
            <span className={category.isPoll ? "chip warn" : "chip soft"}>
              {categoryKind(category)}
            </span>
            {category.acceptsAnswers ? (
              <span className="chip ok">Answers enabled</span>
            ) : null}
          </div>
          <p
            className="t-sm mt-2 break-words"
            style={{ color: "var(--ink-3)" }}
          >
            {category.description || "Start a focused repository conversation."}
          </p>
        </div>
      </div>

      <div className="mt-5 flex flex-wrap items-center gap-2">
        <span className="chip soft">
          <span className="t-num">{formatNumber(category.openCount)}</span> open
        </span>
        <span className="chip soft">
          <span className="t-num">{formatNumber(category.count)}</span> total
        </span>
      </div>

      <div className="mt-5">
        {disabled ? (
          <span className="btn opacity-60" aria-disabled="true">
            Get started
          </span>
        ) : (
          <Link className="btn primary" href={href}>
            Get started
          </Link>
        )}
      </div>
    </article>
  );
}

export function RepositoryDiscussionCategoryChooser({
  creation,
  repository,
}: RepositoryDiscussionCategoryChooserProps) {
  const owner = repository.owner_login;
  const repo = repository.name;
  const disabled = !creation.enabled || !creation.viewer.canCreate;

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/discussions`}
      frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0 space-y-5">
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            New discussion
          </p>
          <h1 className="t-h2 mt-1">Choose a category</h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            Pick the conversation shape that best fits your question, idea, or
            poll.
          </p>
        </section>

        {creation.enabled ? null : (
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
              {creation.disabledReason ??
                "Repository discussions are disabled by organization policy."}
            </p>
          </section>
        )}

        {creation.categories.length ? (
          <section
            aria-label="Discussion categories"
            className="grid gap-4 md:grid-cols-2"
          >
            {creation.categories.map((category) => (
              <CategoryCard
                category={category}
                disabled={disabled}
                key={category.id}
                owner={owner}
                repo={repo}
              />
            ))}
          </section>
        ) : (
          <section className="card grid justify-items-center gap-3 px-6 py-14 text-center">
            <span className="chip soft">No categories</span>
            <h2 className="t-h2">No discussion categories are available.</h2>
            <p className="t-sm max-w-xl" style={{ color: "var(--ink-3)" }}>
              A repository maintainer needs to publish at least one category
              before new discussions can start.
            </p>
          </section>
        )}
      </main>

      <aside className="space-y-4">
        <section className="card p-4">
          <h2 className="t-h3">Before you post</h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Search for an existing discussion first. The next step keeps that
            acknowledgement with the draft.
          </p>
        </section>

        <section className="card p-4">
          <h2 className="t-h3">Community resources</h2>
          <div className="mt-3 grid gap-2">
            {creation.communityLinks.length ? (
              creation.communityLinks.map((link) => (
                <Link
                  className="t-sm hover:underline"
                  href={link.href}
                  key={link.id}
                >
                  {link.label}
                </Link>
              ))
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Community links have not been published for this repository.
              </p>
            )}
          </div>
        </section>
      </aside>
    </RepositoryShell>
  );
}
