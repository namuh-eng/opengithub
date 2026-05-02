export const PROTECTED_PATHS = [
  "/dashboard",
  "/new",
  "/issues",
  "/pulls",
  "/notifications",
  "/search",
  "/explore",
  "/codespaces",
  "/settings",
  "/organizations/new",
] as const;

type RequestUrlParts = {
  url: string;
  nextUrl: {
    pathname: string;
    search: string;
  };
};

export function isProtectedPath(pathname: string): boolean {
  if (
    PROTECTED_PATHS.some(
      (path) => pathname === path || pathname.startsWith(`${path}/`),
    )
  ) {
    return true;
  }

  return (
    /^\/[^/]+\/[^/]+\/settings(?:\/|$)/.test(pathname) ||
    /^\/orgs\/[^/]+\/settings(?:\/|$)/.test(pathname) ||
    /^\/organizations\/[^/]+\/settings(?:\/|$)/.test(pathname)
  );
}

export function preservedNextPath(request: Pick<RequestUrlParts, "nextUrl">) {
  const path = `${request.nextUrl.pathname}${request.nextUrl.search}`;
  return path.startsWith("/") && !path.startsWith("//") ? path : "/dashboard";
}

export function loginRedirectUrl(request: RequestUrlParts) {
  const loginUrl = new URL("/login", request.url);
  loginUrl.searchParams.set("next", preservedNextPath(request));
  return loginUrl;
}
