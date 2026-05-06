import { redirect } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { IssueCreateForm } from "@/components/IssueCreateForm";
import { IssueTemplateChooser } from "@/components/IssueTemplateChooser";
import { RepositoryShell } from "@/components/RepositoryShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryIssueTemplates,
  getSessionAndShellContext,
} from "@/lib/server-session";

type NewRepositoryIssuePageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    body?: string;
    milestone?: string;
    template?: string;
    title?: string;
  }>;
};

export default async function NewRepositoryIssuePage({
  params,
  searchParams,
}: NewRepositoryIssuePageProps) {
  const [{ owner, repo }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const base = `/${ownerLogin}/${repositoryName}`;
  const nextSearch = new URLSearchParams();
  if (query.template) {
    nextSearch.set("template", query.template);
  }
  if (query.title) {
    nextSearch.set("title", query.title);
  }
  if (query.body) {
    nextSearch.set("body", query.body);
  }
  if (query.milestone) {
    nextSearch.set("milestone", query.milestone);
  }
  const nextPath = `${base}/issues/new${nextSearch.size ? `?${nextSearch.toString()}` : ""}`;

  if (!session.authenticated || !session.user) {
    redirect(`/login?next=${encodeURIComponent(nextPath)}`);
  }

  const [repository, templates] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryIssueTemplates(ownerLogin, repositoryName),
  ]);
  const selectedTemplate =
    query.template && query.template !== "blank"
      ? templates.find((template) => template.slug === query.template)
      : null;
  const shouldChooseTemplate = templates.length > 0 && !query.template;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositoryShell
          activePath={`${base}/issues/new`}
          frameClassName="mx-auto max-w-5xl px-6 py-8"
          repository={repository}
        >
          {shouldChooseTemplate ? (
            <IssueTemplateChooser
              cancelHref={`${base}/issues`}
              newIssueHref={`${base}/issues/new`}
              owner={ownerLogin}
              repo={repositoryName}
              templates={templates}
            />
          ) : (
            <IssueCreateForm
              cancelHref={`${base}/issues`}
              defaultAssigneeUserIds={
                selectedTemplate?.defaultAssigneeUserIds ?? []
              }
              defaultLabelIds={selectedTemplate?.defaultLabelIds ?? []}
              defaultMilestoneId={query.milestone ?? null}
              formFields={selectedTemplate?.formFields ?? []}
              initialBody={query.body ?? selectedTemplate?.body ?? ""}
              initialTitle={query.title ?? selectedTemplate?.titlePrefill ?? ""}
              owner={ownerLogin}
              repo={repositoryName}
              templateId={selectedTemplate?.id ?? null}
              templateName={selectedTemplate?.name ?? null}
              templateSlug={selectedTemplate?.slug ?? null}
            />
          )}
        </RepositoryShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
