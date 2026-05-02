import { cookies, headers } from "next/headers";
import { NextResponse } from "next/server";
import { updateUserAppearanceSettingsFromCookie } from "@/lib/api";
import { normalizeFontSize, normalizeTheme } from "@/lib/theme";

export async function PATCH(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  try {
    const settings = await updateUserAppearanceSettingsFromCookie(
      cookie,
      input,
    );
    const response = NextResponse.json(settings);
    response.cookies.set("color_mode", normalizeTheme(settings.theme), {
      httpOnly: false,
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 365,
    });
    response.cookies.set("font_size", normalizeFontSize(settings.fontSize), {
      httpOnly: false,
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 365,
    });
    return response;
  } catch (error) {
    const message =
      error instanceof Error
        ? error.message
        : "Appearance settings update failed";
    return NextResponse.json(
      { error: { code: "validation_failed", message }, status: 422 },
      { status: 422 },
    );
  }
}

export async function POST(request: Request) {
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  const theme = normalizeTheme("theme" in input ? String(input.theme) : null);
  const fontSize = normalizeFontSize(
    "fontSize" in input ? String(input.fontSize) : null,
  );
  const cookieStore = await cookies();
  cookieStore.set("color_mode", theme, {
    httpOnly: false,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 24 * 365,
  });
  cookieStore.set("font_size", fontSize, {
    httpOnly: false,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 24 * 365,
  });
  return NextResponse.json({ theme, fontSize });
}
