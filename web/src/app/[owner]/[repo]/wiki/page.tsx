import { AppShell } from "@/components/AppShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWikiPage as RepositoryWikiView } from "@/components/RepositoryWikiPage";
import {
  getRepository,
  getRepositoryWiki,
  getSession,
} from "@/lib/server-session";

type RepositoryWikiPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositoryWikiPage({
  params,
}: RepositoryWikiPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, wikiResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryWiki(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryWikiView repository={repository} wikiResult={wikiResult} />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
