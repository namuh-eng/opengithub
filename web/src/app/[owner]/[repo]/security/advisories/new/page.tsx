import { AppShell } from "@/components/AppShell";
import { RepositorySecurityAdvisoryCreatePage as RepositorySecurityAdvisoryCreateView } from "@/components/RepositorySecurityAdvisoryCreatePage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { getRepository, getSession } from "@/lib/server-session";

type RepositorySecurityAdvisoryNewPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySecurityAdvisoryNewPage({
  params,
}: RepositorySecurityAdvisoryNewPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecurityAdvisoryCreateView repository={repository} />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
