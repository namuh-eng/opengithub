import { AppShell } from "@/components/AppShell";
import { RepositoryPulsePage as RepositoryPulseView } from "@/components/RepositoryPulsePage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryPulse,
  getSession,
} from "@/lib/server-session";

type RepositoryPulsePageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{ period?: string }>;
};

export default async function RepositoryPulsePage({
  params,
  searchParams,
}: RepositoryPulsePageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, pulseResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryPulse(ownerLogin, repositoryName, {
      period: query?.period ?? null,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryPulseView
          pulseResult={pulseResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
