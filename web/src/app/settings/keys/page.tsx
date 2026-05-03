import { DeveloperKeysPage } from "@/components/DeveloperKeysPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getKeySettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function KeySettingsPage() {
  const [{ session, shellContext }, keySettings] = await Promise.all([
    getSessionAndShellContext(),
    getKeySettings(),
  ]);

  return (
    <SettingsShell
      activeSection="keys"
      eyebrow="Developer settings"
      session={session}
      shellContext={shellContext}
      title="SSH keys"
    >
      <DeveloperKeysPage
        keySettings={keySettings}
        showHeading={false}
        userEmail={session.user?.email}
      />
    </SettingsShell>
  );
}
