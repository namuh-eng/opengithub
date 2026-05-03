import { DeveloperTokensPage } from "@/components/DeveloperTokensPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getPersonalAccessTokenList,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function SettingsTokensRoute() {
  const [{ session, shellContext }, tokenList] = await Promise.all([
    getSessionAndShellContext(),
    getPersonalAccessTokenList(),
  ]);

  return (
    <SettingsShell
      activeSection="tokens"
      eyebrow="Developer settings"
      session={session}
      shellContext={shellContext}
      title="Personal access tokens"
    >
      <DeveloperTokensPage showHeading={false} tokenList={tokenList} />
    </SettingsShell>
  );
}
