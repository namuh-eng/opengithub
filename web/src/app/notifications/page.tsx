import { headers } from "next/headers";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { NotificationsInboxPage } from "@/components/NotificationsInboxPage";
import {
  getNotificationsFromCookie,
  type NotificationInboxQuery,
} from "@/lib/api";
import { getSessionAndShellContext } from "@/lib/server-session";

type NotificationsPageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined): string | undefined {
  return Array.isArray(value) ? value[0] : value;
}

export default async function NotificationsPage({
  searchParams,
}: NotificationsPageProps) {
  const requestHeaders = await headers();
  const params = (await searchParams) ?? {};
  const query: NotificationInboxQuery = {
    q: firstParam(params.q),
    folder: firstParam(params.folder),
    tab: firstParam(params.tab),
    sort: firstParam(params.sort),
    group: firstParam(params.group),
    repo: firstParam(params.repo),
    page: firstParam(params.page),
    pageSize: firstParam(params.pageSize),
  };
  const [{ session, shellContext }, notifications] = await Promise.all([
    getSessionAndShellContext(),
    getNotificationsFromCookie(requestHeaders.get("cookie"), query),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <NotificationsInboxPage view={notifications} />
      </AppShellFrame>
    </AppShell>
  );
}
