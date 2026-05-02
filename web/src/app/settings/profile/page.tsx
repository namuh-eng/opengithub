import { PersonalProfileSettingsForm } from "@/components/PersonalProfileSettingsForm";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getPersonalProfileSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function ProfileSettingsPage() {
  const [{ session, shellContext }, settings] = await Promise.all([
    getSessionAndShellContext(),
    getPersonalProfileSettings(),
  ]);

  return (
    <SettingsShell
      activeSection="profile"
      eyebrow="Settings"
      session={session}
      shellContext={shellContext}
      title="Public profile"
    >
      {settings ? (
        <PersonalProfileSettingsForm initialSettings={settings} />
      ) : (
        <div className="card p-6">
          <p className="t-label">Unavailable</p>
          <h3 className="t-h2 mt-2">Profile settings could not be loaded</h3>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Refresh the page after signing in again. No profile changes were
            made.
          </p>
        </div>
      )}
    </SettingsShell>
  );
}
