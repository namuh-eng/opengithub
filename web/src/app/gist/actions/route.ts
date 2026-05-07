import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  createGistFromCookie,
  forkGistFromCookie,
  starGistFromCookie,
  updateGistFromCookie,
} from "@/lib/api";

function filesFromForm(form: FormData) {
  const raw = String(form.get("filesJson") ?? "[]");
  const parsed = JSON.parse(raw) as Array<{
    filename?: string;
    content?: string;
  }>;
  return parsed.map((file) => ({
    filename: String(file.filename ?? ""),
    content: String(file.content ?? ""),
  }));
}

function redirectTo(request: Request, path: string, status = 303) {
  return NextResponse.redirect(new URL(path, request.url), status);
}

export async function POST(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const form = await request.formData();
  const intent = String(form.get("intent") ?? "create");
  try {
    if (intent === "star" || intent === "unstar") {
      const gistId = String(form.get("gistId"));
      await starGistFromCookie(cookie, gistId, intent === "star");
      return redirectTo(request, `/gist/${encodeURIComponent(gistId)}`);
    }
    if (intent === "fork") {
      const gistId = String(form.get("gistId"));
      const fork = await forkGistFromCookie(cookie, gistId);
      return redirectTo(request, fork.href);
    }
    const payload = {
      description: String(form.get("description") ?? ""),
      isPublic: String(form.get("visibility") ?? "public") === "public",
      files: filesFromForm(form),
    };
    if (intent === "update") {
      const gistId = String(form.get("gistId"));
      const gist = await updateGistFromCookie(cookie, gistId, payload);
      return redirectTo(request, gist.href);
    }
    const gist = await createGistFromCookie(cookie, payload);
    return redirectTo(request, gist.href);
  } catch {
    return NextResponse.json(
      { error: { code: "gist_action_failed", message: "Gist action failed" } },
      { status: 422 },
    );
  }
}
