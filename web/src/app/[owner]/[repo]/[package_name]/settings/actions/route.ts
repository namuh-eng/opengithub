import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  mutateUserPackageSettingsFromCookie,
  type PackageSettingsMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    package_name: string;
  }>;
};

export async function PATCH(request: Request, { params }: RouteContext) {
  const { owner, repo: packageType, package_name: packageName } = await params;
  const cookie = (await headers()).get("cookie");
  const mutation = (await request
    .json()
    .catch(() => null)) as PackageSettingsMutation | null;

  if (!mutation?.action) {
    return NextResponse.json(
      {
        error: {
          code: "package_settings_invalid_action",
          message: "Package settings action is required.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  try {
    const settings = await mutateUserPackageSettingsFromCookie(
      cookie,
      owner,
      packageType,
      packageName,
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    return NextResponse.json(
      {
        error: {
          code: "package_settings_update_failed",
          message:
            error instanceof Error
              ? error.message
              : "Package settings update failed.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }
}
