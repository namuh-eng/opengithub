import { AppearanceSettingsForm } from "@/components/AppearanceSettingsForm";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getAppearanceSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function AppearanceSettingsPage() {
  const [{ session, shellContext }, settings] = await Promise.all([
    getSessionAndShellContext(),
    getAppearanceSettings(),
  ]);

  return (
    <SettingsShell
      activeSection="appearance"
      eyebrow="Settings"
      session={session}
      shellContext={shellContext}
      title="Appearance"
    >
      {settings ? (
        <AppearanceSettingsForm initialSettings={settings} />
      ) : (
        <div className="card p-6">
          <p className="t-label">Unavailable</p>
          <h3 className="t-h2 mt-2">Appearance settings could not be loaded</h3>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Refresh after signing in again. The Editorial system theme remains
            active until preferences are reachable.
          </p>
        </div>
      )}
    </SettingsShell>
  );
}
