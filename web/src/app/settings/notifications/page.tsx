import { NotificationFilterSettingsPage } from "@/components/NotificationFilterSettingsPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getNotificationFilterSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function NotificationSettingsPage() {
  const [{ session, shellContext }, filterSettings] = await Promise.all([
    getSessionAndShellContext(),
    getNotificationFilterSettings(),
  ]);

  return (
    <SettingsShell
      activeSection="notifications"
      eyebrow="Settings"
      session={session}
      shellContext={shellContext}
      title="Notifications"
    >
      {"error" in filterSettings ? (
        <div className="card p-6">
          <p className="t-label">Unavailable</p>
          <h3 className="t-h2 mt-2">
            Notification filters could not be loaded
          </h3>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Refresh after signing in again. No filter changes were made.
          </p>
        </div>
      ) : (
        <NotificationFilterSettingsPage initialSettings={filterSettings} />
      )}
    </SettingsShell>
  );
}
