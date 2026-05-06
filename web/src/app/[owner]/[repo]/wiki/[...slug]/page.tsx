import { AppShell } from "@/components/AppShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWikiPage as RepositoryWikiView } from "@/components/RepositoryWikiPage";
import {
  getRepository,
  getRepositoryWiki,
  getSession,
} from "@/lib/server-session";

type RepositoryWikiSlugPageProps = {
  params: Promise<{ owner: string; repo: string; slug: string[] }>;
};

function decodePathSegment(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function wikiSlugFromParams(slug: string[]) {
  return slug.map(decodePathSegment).filter(Boolean).join("/");
}

export default async function RepositoryWikiSlugPage({
  params,
}: RepositoryWikiSlugPageProps) {
  const [{ owner, repo, slug }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodePathSegment(owner);
  const repositoryName = decodePathSegment(repo);
  const wikiSlug = wikiSlugFromParams(slug);
  const [repository, wikiResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryWiki(ownerLogin, repositoryName, wikiSlug),
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
