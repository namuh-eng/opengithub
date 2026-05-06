import { AccountSessionsPage } from "@/components/AccountSessionsPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getAccountSessions,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function SecuritySessionSettingsPage() {
  const [{ session, shellContext }, sessionsResult] = await Promise.all([
    getSessionAndShellContext(),
    getAccountSessions(),
  ]);

  return (
    <SettingsShell
      activeSection="security"
      eyebrow="Personal settings"
      session={session}
      shellContext={shellContext}
      title="Sessions"
    >
      <AccountSessionsPage sessionsResult={sessionsResult} />
    </SettingsShell>
  );
}
