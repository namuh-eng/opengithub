import { headers } from "next/headers";
import { AdminSearchIndexPage } from "@/components/AdminSearchIndexPage";
import { AppShell } from "@/components/AppShell";
import {
  getAppShellContextFromCookie,
  getSearchIndexStatusFromCookie,
  getSessionFromHeaders,
} from "@/lib/api";

export default async function AdminSearchPage() {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const [session, shellContext, status] = await Promise.all([
    getSessionFromHeaders(requestHeaders),
    getAppShellContextFromCookie(cookie),
    getSearchIndexStatusFromCookie(cookie),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AdminSearchIndexPage status={status} />
    </AppShell>
  );
}
