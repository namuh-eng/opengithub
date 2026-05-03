import { AppShell } from "@/components/AppShell";
import { RepositoryReleaseFormPage } from "@/components/RepositoryReleaseFormPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryReleaseManagementContext,
  getSession,
} from "@/lib/server-session";

type NewReleasePageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function NewReleasePage({ params }: NewReleasePageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const context = repository
    ? await getRepositoryReleaseManagementContext(ownerLogin, repositoryName)
    : null;

  return (
    <AppShell session={session}>
      {repository && context ? (
        <RepositoryReleaseFormPage
          context={context}
          mode="new"
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
