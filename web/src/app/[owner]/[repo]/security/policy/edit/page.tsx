import { AppShell } from "@/components/AppShell";
import { RepositorySecurityPolicyEditorPage } from "@/components/RepositorySecurityPolicyEditorPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecurityPolicy,
  getSession,
} from "@/lib/server-session";

type RepositorySecurityPolicyEditPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySecurityPolicyEditPage({
  params,
}: RepositorySecurityPolicyEditPageProps) {
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
        <RepositorySecurityPolicyEditorPage
          policyResult={policyResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
