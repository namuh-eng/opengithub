import { headers } from "next/headers";
import { NextResponse } from "next/server";
import {
  mutateOrganizationPackageSettingsFromCookie,
  type PackageSettingsMutation,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    org: string;
    package_type: string;
    package_name: string;
  }>;
};

export async function PATCH(request: Request, { params }: RouteContext) {
  const {
    org,
    package_type: packageType,
    package_name: packageName,
  } = await params;
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
    const settings = await mutateOrganizationPackageSettingsFromCookie(
      cookie,
      org,
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
