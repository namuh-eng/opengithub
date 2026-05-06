import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  type AppearanceFontSize,
  type AppearanceTheme,
  updateAppearanceSettingsFromCookie,
} from "@/lib/api";

const THEMES = new Set<AppearanceTheme>([
  "light",
  "dark",
  "system",
  "dark_dimmed",
  "dark_high_contrast",
]);
const FONT_SIZES = new Set<AppearanceFontSize>(["small", "default", "large"]);

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

  const body = input as {
    theme?: AppearanceTheme;
    fontSize?: AppearanceFontSize;
  };
  if (body.theme && !THEMES.has(body.theme)) {
    return NextResponse.json(
      {
        error: { code: "validation_failed", message: "Theme is not valid." },
        status: 422,
      },
      { status: 422 },
    );
  }
  if (body.fontSize && !FONT_SIZES.has(body.fontSize)) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Font size is not valid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateAppearanceSettingsFromCookie(cookie, body);
    const response = NextResponse.json(settings);
    response.cookies.set("color_mode", settings.theme, {
      httpOnly: false,
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 365,
    });
    response.cookies.set("font_size", settings.fontSize, {
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
