import { headers } from "next/headers";
import { redirect } from "next/navigation";
import { markNotificationReadFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ notificationId: string }>;
};

function safeNext(value: string | null): string {
  if (!value?.startsWith("/") || value.startsWith("//")) {
    return "/notifications";
  }
  return value;
}

export async function GET(request: Request, { params }: RouteContext) {
  const { notificationId } = await params;
  const requestHeaders = await headers();
  await markNotificationReadFromCookie(
    requestHeaders.get("cookie"),
    notificationId,
  );
  redirect(safeNext(new URL(request.url).searchParams.get("next")));
}
