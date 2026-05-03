import { NotificationDeliverySettingsPage } from "@/components/NotificationDeliverySettingsPage";
import { NotificationFilterSettingsPage } from "@/components/NotificationFilterSettingsPage";
import { SettingsShell } from "@/components/SettingsShell";
import {
  getNotificationDeliverySettings,
  getNotificationFilterSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

export default async function NotificationSettingsPage() {
  const [{ session, shellContext }, deliverySettings, filterSettings] =
    await Promise.all([
      getSessionAndShellContext(),
      getNotificationDeliverySettings(),
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
      {"error" in deliverySettings || "error" in filterSettings ? (
        <div className="card p-6">
          <p className="t-label">Unavailable</p>
          <h3 className="t-h2 mt-2">
            Notification settings could not be loaded
          </h3>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Refresh after signing in again. No notification preference changes
            were made.
          </p>
        </div>
      ) : (
        <div className="grid gap-6">
          <NotificationDeliverySettingsPage
            initialSettings={deliverySettings}
          />
          <div id="custom-routing">
            <NotificationFilterSettingsPage initialSettings={filterSettings} />
          </div>
        </div>
      )}
    </SettingsShell>
  );
}
