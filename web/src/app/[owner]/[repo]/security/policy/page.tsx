import { AppShell } from "@/components/AppShell";
import { RepositorySecurityPolicyPage as RepositorySecurityPolicyView } from "@/components/RepositorySecurityPolicyPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecurityPolicy,
  getSession,
} from "@/lib/server-session";

type RepositorySecurityPolicyPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySecurityPolicyPage({
  params,
}: RepositorySecurityPolicyPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, policyResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecurityPolicy(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecurityPolicyView
          policyResult={policyResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
