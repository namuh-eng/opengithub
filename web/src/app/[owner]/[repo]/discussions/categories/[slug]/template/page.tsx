import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionCategoryTemplatePage } from "@/components/RepositoryDiscussionCategoryTemplatePage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDiscussionCategoryTemplate,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryDiscussionCategoryTemplateRouteProps = {
  params: Promise<{ owner: string; repo: string; slug: string }>;
};

export default async function RepositoryDiscussionCategoryTemplateRoute({
  params,
}: RepositoryDiscussionCategoryTemplateRouteProps) {
  const { owner, repo, slug } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedCategoryId = decodeURIComponent(slug);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, template] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussionCategoryTemplate(
      ownerLogin,
      repositoryName,
      decodedCategoryId,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositoryDiscussionCategoryTemplatePage
          repository={repository}
          template={template}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
