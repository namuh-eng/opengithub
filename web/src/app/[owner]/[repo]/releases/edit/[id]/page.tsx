import { AppShell } from "@/components/AppShell";
import { RepositoryReleaseFormPage } from "@/components/RepositoryReleaseFormPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryReleaseManagementContext,
  getSession,
} from "@/lib/server-session";

type EditReleasePageProps = {
  params: Promise<{ owner: string; repo: string; id: string }>;
};

export default async function EditReleasePage({
  params,
}: EditReleasePageProps) {
  const [{ owner, repo, id }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const releaseId = decodeURIComponent(id);
  const repository = await getRepository(ownerLogin, repositoryName);
  const context = repository
    ? await getRepositoryReleaseManagementContext(
        ownerLogin,
        repositoryName,
        releaseId,
      )
    : null;

  return (
    <AppShell session={session}>
      {repository && context ? (
        <RepositoryReleaseFormPage
          context={context}
          mode="edit"
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
