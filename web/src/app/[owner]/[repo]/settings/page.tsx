import { AppShell } from "@/components/AppShell";
import { RepositorySettingsOverview } from "@/components/RepositorySettingsOverview";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

function isErrorEnvelope(
  value: unknown,
): value is { error: { code: string; message: string } } {
  return Boolean(
    value &&
      typeof value === "object" &&
      "error" in value &&
      typeof (value as { error?: unknown }).error === "object",
  );
}

export default async function RepositorySettingsPage({
  params,
}: RepositorySettingsPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settings] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="general"
          repository={repository}
          title="General"
        >
          {isErrorEnvelope(settings) ? (
            <div className="card p-5">
              <span className="chip err">{settings.error.code}</span>
              <p className="mt-3 t-body" style={{ color: "var(--ink-2)" }}>
                {settings.error.message}
              </p>
            </div>
          ) : (
            <RepositorySettingsOverview initialSettings={settings} />
          )}
        </RepositorySettingsShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
