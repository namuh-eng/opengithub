export type NavigationKind =
  | "primary"
  | "create"
  | "settings"
  | "repository"
  | "profile"
  | "organization"
  | "search";

export type NavigationItem = {
  href: string;
  label: string;
  kind: NavigationKind;
  description?: string;
  protected: boolean;
};

export type SettingsSection = NavigationItem & {
  section: string;
};

export type RepositoryTab = NavigationItem & {
  segment: string;
};

export type RepositorySettingsSection = NavigationItem & {
  section: string;
  hrefSuffix: string;
};

export type OrganizationSettingsSection = NavigationItem & {
  section: string;
  group: "general" | "access" | "integrations" | "danger";
  disabled?: boolean;
};

export type QueryTab = {
  label: string;
  value: string;
  description: string;
};

export const GLOBAL_NAV_ITEMS = [
  {
    href: "/dashboard",
    label: "Dashboard",
    kind: "primary",
    description: "Home feed and repository overview",
    protected: true,
  },
  {
    href: "/pulls",
    label: "Pull requests",
    kind: "primary",
    description: "Review requests across repositories",
    protected: true,
  },
  {
    href: "/issues",
    label: "Issues",
    kind: "primary",
    description: "Assigned, mentioned, and subscribed issues",
    protected: true,
  },
  {
    href: "/notifications",
    label: "Notifications",
    kind: "primary",
    description: "Unread and done inbox triage",
    protected: true,
  },
  {
    href: "/search",
    label: "Search",
    kind: "search",
    description: "Search repositories, code, issues, and people",
    protected: true,
  },
  {
    href: "/explore",
    label: "Explore",
    kind: "primary",
    description: "Discover repositories and activity",
    protected: true,
  },
  {
    href: "/codespaces",
    label: "Codespaces",
    kind: "primary",
    description: "Cloud development environments",
    protected: true,
  },
] as const satisfies readonly NavigationItem[];

export const CREATE_NAV_ITEMS = [
  {
    href: "/new",
    label: "New repository",
    kind: "create",
    description: "Create a repository owned by you or an organization",
    protected: true,
  },
  {
    href: "/new/import",
    label: "Import repository",
    kind: "create",
    description: "Import an existing Git repository",
    protected: true,
  },
  {
    href: "/organizations/new",
    label: "New organization",
    kind: "create",
    description: "Create a shared organization workspace",
    protected: true,
  },
] as const satisfies readonly NavigationItem[];

export const SETTINGS_NAV_ITEMS = [
  {
    href: "/settings/profile",
    label: "Profile",
    section: "profile",
    kind: "settings",
    description: "Public identity and profile details",
    protected: true,
  },
  {
    href: "/settings/account",
    label: "Account",
    section: "account",
    kind: "settings",
    description: "Username, export, and account controls",
    protected: true,
  },
  {
    href: "/settings/emails",
    label: "Emails",
    section: "emails",
    kind: "settings",
    description: "Primary Google email and notification addresses",
    protected: true,
  },
  {
    href: "/settings/notifications",
    label: "Notifications",
    section: "notifications",
    kind: "settings",
    description: "Web and email notification preferences",
    protected: true,
  },
  {
    href: "/settings/appearance",
    label: "Appearance",
    section: "appearance",
    kind: "settings",
    description: "Theme and accessibility preferences",
    protected: true,
  },
  {
    href: "/settings/security",
    label: "Security",
    section: "security",
    kind: "settings",
    description: "Sessions, providers, and security log",
    protected: true,
  },
  {
    href: "/settings/sessions",
    label: "Sessions",
    section: "sessions",
    kind: "settings",
    description: "Signed-in browser sessions",
    protected: true,
  },
  {
    href: "/settings/keys",
    label: "Keys",
    section: "keys",
    kind: "settings",
    description: "SSH and signing keys",
    protected: true,
  },
  {
    href: "/settings/tokens",
    label: "Tokens",
    section: "tokens",
    kind: "settings",
    description: "Personal access tokens for Git and API access",
    protected: true,
  },
] as const satisfies readonly SettingsSection[];

export const REPOSITORY_TABS = [
  {
    href: "",
    label: "Code",
    segment: "",
    kind: "repository",
    protected: false,
  },
  {
    href: "/issues",
    label: "Issues",
    segment: "issues",
    kind: "repository",
    protected: false,
  },
  {
    href: "/pulls",
    label: "Pull requests",
    segment: "pulls",
    kind: "repository",
    protected: false,
  },
  {
    href: "/actions",
    label: "Actions",
    segment: "actions",
    kind: "repository",
    protected: false,
  },
  {
    href: "/projects",
    label: "Projects",
    segment: "projects",
    kind: "repository",
    protected: false,
  },
  {
    href: "/wiki",
    label: "Wiki",
    segment: "wiki",
    kind: "repository",
    protected: false,
  },
  {
    href: "/security",
    label: "Security",
    segment: "security",
    kind: "repository",
    protected: false,
  },
  {
    href: "/pulse",
    label: "Insights",
    segment: "pulse",
    kind: "repository",
    protected: false,
  },
  {
    href: "/settings",
    label: "Settings",
    segment: "settings",
    kind: "repository",
    protected: true,
  },
] as const satisfies readonly RepositoryTab[];

export const REPOSITORY_SETTINGS_NAV_ITEMS = [
  {
    href: "",
    hrefSuffix: "/settings",
    label: "General",
    section: "general",
    kind: "settings",
    description: "Repository name, visibility, and default branch",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/access",
    label: "Access",
    section: "access",
    kind: "settings",
    description: "Collaborators, teams, and repository permissions",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/branches",
    label: "Branches",
    section: "branches",
    kind: "settings",
    description: "Default branch and branch protection rules",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/actions",
    label: "Actions",
    section: "actions",
    kind: "settings",
    description: "Workflow permissions and runner policy",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/hooks",
    label: "Webhooks",
    section: "hooks",
    kind: "settings",
    description: "Repository webhook endpoints and deliveries",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/pages",
    label: "Pages",
    section: "pages",
    kind: "settings",
    description: "Static site publishing and custom domains",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/secrets",
    label: "Secrets",
    section: "secrets",
    kind: "settings",
    description: "Actions secrets and environment variables",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/tags",
    label: "Tags",
    section: "tags",
    kind: "settings",
    description: "Protected tags and release rules",
    protected: true,
  },
  {
    href: "",
    hrefSuffix: "/settings/security",
    label: "Security analysis",
    section: "security",
    kind: "settings",
    description: "Security features, alerts, and audit controls",
    protected: true,
  },
] as const satisfies readonly RepositorySettingsSection[];

export const ORGANIZATION_SETTINGS_NAV_ITEMS = [
  {
    href: "/settings/profile",
    label: "Profile",
    section: "profile",
    group: "general",
    kind: "settings",
    description: "Public organization profile and contact fields",
    protected: true,
  },
  {
    href: "/settings/member-privileges",
    label: "Member privileges",
    section: "member-privileges",
    group: "access",
    kind: "settings",
    description: "Repository creation and member policy defaults",
    protected: true,
  },
  {
    href: "/settings/teams",
    label: "Teams",
    section: "teams",
    group: "access",
    kind: "settings",
    description: "Team directory and repository access",
    protected: true,
  },
  {
    href: "/settings/hooks",
    label: "Webhooks",
    section: "hooks",
    group: "integrations",
    kind: "settings",
    description: "Organization webhook endpoints",
    protected: true,
  },
  {
    href: "/settings/packages",
    label: "Packages",
    section: "packages",
    group: "integrations",
    kind: "settings",
    description: "Package publishing defaults",
    protected: true,
  },
  {
    href: "/settings/billing",
    label: "Billing",
    section: "billing",
    group: "general",
    kind: "settings",
    description: "Billing is outside this clone's current scope",
    protected: true,
    disabled: true,
  },
  {
    href: "/settings/danger",
    label: "Danger zone",
    section: "danger",
    group: "danger",
    kind: "settings",
    description: "Rename, archive, and delete guardrails",
    protected: true,
  },
] as const satisfies readonly OrganizationSettingsSection[];

export const PROFILE_TABS = [
  {
    label: "Overview",
    value: "overview",
    description: "Profile summary and contribution highlights",
  },
  {
    label: "Repositories",
    value: "repositories",
    description: "Public and visible repositories owned by this account",
  },
  {
    label: "Projects",
    value: "projects",
    description: "Project boards connected to this account",
  },
  {
    label: "Packages",
    value: "packages",
    description: "Published packages",
  },
  {
    label: "Stars",
    value: "stars",
    description: "Starred repositories",
  },
] as const satisfies readonly QueryTab[];

export const ORGANIZATION_TABS = [
  {
    label: "Overview",
    value: "overview",
    description: "Organization summary and pinned repositories",
  },
  {
    label: "Repositories",
    value: "repositories",
    description: "Repositories owned by this organization",
  },
  {
    label: "Projects",
    value: "projects",
    description: "Organization planning surfaces",
  },
  {
    label: "Packages",
    value: "packages",
    description: "Packages published by this organization",
  },
  {
    label: "People",
    value: "people",
    description: "Organization members and owners",
  },
  {
    label: "Teams",
    value: "teams",
    description: "Team directories and access groups",
  },
] as const satisfies readonly QueryTab[];

export const SEARCH_TABS = [
  {
    label: "Repositories",
    value: "repositories",
    description: "Repository name, description, and topic matches",
  },
  {
    label: "Code",
    value: "code",
    description: "Indexed file content and symbols",
  },
  {
    label: "Issues",
    value: "issues",
    description: "Issue titles, bodies, labels, and comments",
  },
  {
    label: "Pull requests",
    value: "pull_requests",
    description: "Pull request titles, branches, and review text",
  },
  {
    label: "Commits",
    value: "commits",
    description: "Commit messages and authors",
  },
  {
    label: "Users",
    value: "users",
    description: "People using opengithub",
  },
  {
    label: "Organizations",
    value: "organizations",
    description: "Organization profiles and teams",
  },
  {
    label: "Discussions",
    value: "discussions",
    description: "Repository discussions once discussion indexing ships",
  },
] as const satisfies readonly QueryTab[];

export type SearchModalAction =
  | {
      href: string;
      kind: "navigate" | "submit_search";
    }
  | {
      kind: "replace_token";
      nextQuery: string;
    }
  | {
      kind: "open_saved_search_dialog";
    };

export function searchHref(
  query: string,
  resultType = "repositories",
  extraParams: Record<string, string | null | undefined> = {},
) {
  const params = new URLSearchParams();
  const trimmedQuery = query.trim();
  if (trimmedQuery) {
    params.set("q", trimmedQuery);
  }
  params.set("type", resultType);
  for (const [key, value] of Object.entries(extraParams)) {
    if (value?.trim()) {
      params.set(key, value.trim());
    }
  }
  return `/search?${params.toString()}`;
}

export function addSearchQualifier(
  query: string,
  qualifier: string,
  value: string,
) {
  return `${query.trim()} ${qualifier}:${quoteSearchQualifierValue(value)}`.trim();
}

export function removeSearchQualifier(
  query: string,
  qualifier: string,
  value: string,
) {
  return removeCodeSearchQualifier(query, qualifier, value);
}

export function toggleSearchQualifier(
  query: string,
  qualifier: string,
  value: string,
) {
  const removed = removeSearchQualifier(query, qualifier, value);
  return removed === query.trim()
    ? addSearchQualifier(query, qualifier, value)
    : removed;
}

export function replaceSearchQueryToken(
  query: string,
  replaceFrom: number,
  replaceTo: number,
  replacement: string,
) {
  const nextQuery = `${query.slice(0, replaceFrom)}${replacement}${query.slice(replaceTo)}`;
  return nextQuery.endsWith(" ") ? nextQuery : `${nextQuery} `;
}

export function searchModalActionHref(
  action: SearchModalAction,
  fallbackQuery: string,
) {
  if (action.kind === "navigate" || action.kind === "submit_search") {
    return action.href;
  }
  if (action.kind === "replace_token") {
    return searchHref(action.nextQuery || fallbackQuery);
  }
  return "/search?saved=1";
}

export type JumpSuggestionKind =
  | "repository"
  | "organization"
  | "team"
  | "create"
  | "search";

export type JumpSuggestion = {
  id: string;
  kind: JumpSuggestionKind;
  label: string;
  description: string;
  href: string;
  section: "Jump to" | "Create" | "Search";
};

export function navigationHrefs() {
  return [
    ...GLOBAL_NAV_ITEMS.map((item) => item.href),
    ...CREATE_NAV_ITEMS.map((item) => item.href),
    ...SETTINGS_NAV_ITEMS.map((item) => item.href),
  ];
}

function tabValue<T extends QueryTab>(
  tabs: readonly T[],
  value: string | null,
): string {
  if (value && tabs.some((tab) => tab.value === value)) {
    return value;
  }

  return tabs[0].value;
}

function queryTabHref(
  basePath: string,
  paramName: string,
  value: string,
  preservedParams: Record<string, string | null | undefined> = {},
) {
  const params = new URLSearchParams();
  for (const [key, paramValue] of Object.entries(preservedParams)) {
    if (paramValue?.trim()) {
      params.set(key, paramValue.trim());
    }
  }
  params.set(paramName, value);
  return `${basePath}?${params.toString()}`;
}

export function activeProfileTab(value: string | null | undefined) {
  return tabValue(PROFILE_TABS, value ?? null);
}

export function profileTabHref(owner: string, tabValueName: string) {
  return queryTabHref(`/${encodeURIComponent(owner)}`, "tab", tabValueName);
}

export function activeOrganizationTab(value: string | null | undefined) {
  return tabValue(ORGANIZATION_TABS, value ?? null);
}

export function organizationHref(org: string) {
  return `/orgs/${encodeURIComponent(org)}`;
}

export function organizationTabHref(org: string, tabValueName: string) {
  return queryTabHref(organizationHref(org), "tab", tabValueName);
}

export type OrganizationRepositoryListFilters = {
  query?: string | null;
  repositoryType?: string | null;
  language?: string | null;
  sort?: string | null;
  density?: string | null;
  page?: number | string | null;
  pageSize?: number | string | null;
};

export function organizationRepositoryListHref(
  org: string,
  filters: OrganizationRepositoryListFilters = {},
  overrides: Partial<
    Record<
      "q" | "type" | "language" | "sort" | "density" | "page",
      string | null
    >
  > & { pageSize?: string | null } = {},
) {
  const params = new URLSearchParams();
  const nextQuery =
    overrides.q === undefined ? filters.query : overrides.q?.trim() || null;
  const nextType =
    overrides.type === undefined
      ? filters.repositoryType
      : overrides.type?.trim() || "all";
  const nextLanguage =
    overrides.language === undefined
      ? filters.language
      : overrides.language?.trim() || null;
  const nextSort =
    overrides.sort === undefined ? filters.sort : overrides.sort?.trim() || "";
  const nextDensity =
    overrides.density === undefined
      ? filters.density
      : overrides.density?.trim() || "comfortable";
  const nextPage =
    overrides.page === undefined
      ? filters.page
      : overrides.page?.trim() || null;
  const nextPageSize =
    overrides.pageSize === undefined
      ? filters.pageSize
      : overrides.pageSize?.trim() || null;

  if (nextQuery?.trim()) {
    params.set("q", nextQuery.trim());
  }
  if (nextType?.trim() && nextType !== "all") {
    params.set("type", nextType.trim());
  }
  if (nextLanguage?.trim()) {
    params.set("language", nextLanguage.trim());
  }
  if (nextSort?.trim() && nextSort !== "updated-desc") {
    params.set("sort", nextSort.trim());
  }
  if (nextDensity?.trim() && nextDensity !== "comfortable") {
    params.set("density", nextDensity.trim());
  }
  if (nextPage && String(nextPage) !== "1") {
    params.set("page", String(nextPage));
  }
  if (nextPageSize && String(nextPageSize) !== "30") {
    params.set("pageSize", String(nextPageSize));
  }

  const query = params.toString();
  return `${organizationHref(org)}/repositories${query ? `?${query}` : ""}`;
}

export type OrganizationPeopleListFilters = {
  query?: string | null;
  page?: number | string | null;
  pageSize?: number | string | null;
};

export function organizationPeopleListHref(
  org: string,
  filters: OrganizationPeopleListFilters = {},
  overrides: Partial<Record<"q" | "page", string | null>> & {
    pageSize?: string | null;
  } = {},
) {
  const params = new URLSearchParams();
  const nextQuery =
    overrides.q === undefined ? filters.query : overrides.q?.trim() || null;
  const nextPage =
    overrides.page === undefined
      ? filters.page
      : overrides.page?.trim() || null;
  const nextPageSize =
    overrides.pageSize === undefined
      ? filters.pageSize
      : overrides.pageSize?.trim() || null;

  if (nextQuery?.trim()) {
    params.set("q", nextQuery.trim());
  }
  if (nextPage && String(nextPage) !== "1") {
    params.set("page", String(nextPage));
  }
  if (nextPageSize && String(nextPageSize) !== "30") {
    params.set("pageSize", String(nextPageSize));
  }

  const query = params.toString();
  return `${organizationHref(org)}/people${query ? `?${query}` : ""}`;
}

export type OwnerPackageListFilters = {
  query?: string | null;
  type?: string | null;
  visibility?: string | null;
  sort?: string | null;
  artifactTab?: string | null;
  page?: number | string | null;
  pageSize?: number | string | null;
};

export function ownerPackagesHref(
  ownerKind: "user" | "organization",
  owner: string,
  filters: OwnerPackageListFilters = {},
  overrides: Partial<
    Record<
      "q" | "type" | "visibility" | "sort" | "artifactTab" | "page",
      string | null
    >
  > & { pageSize?: string | null } = {},
) {
  const params = new URLSearchParams();
  const nextQuery =
    overrides.q === undefined ? filters.query : overrides.q?.trim() || null;
  const nextType =
    overrides.type === undefined
      ? filters.type
      : overrides.type?.trim() || "all";
  const nextVisibility =
    overrides.visibility === undefined
      ? filters.visibility
      : overrides.visibility?.trim() || "all";
  const nextSort =
    overrides.sort === undefined
      ? filters.sort
      : overrides.sort?.trim() || "downloads-desc";
  const nextArtifactTab =
    overrides.artifactTab === undefined
      ? filters.artifactTab
      : overrides.artifactTab?.trim() || "packages";
  const nextPage =
    overrides.page === undefined
      ? filters.page
      : overrides.page?.trim() || null;
  const nextPageSize =
    overrides.pageSize === undefined
      ? filters.pageSize
      : overrides.pageSize?.trim() || null;

  if (ownerKind === "user") {
    params.set("tab", "packages");
  }
  if (nextQuery?.trim()) {
    params.set("q", nextQuery.trim());
  }
  if (nextType?.trim() && nextType !== "all") {
    params.set("type", nextType.trim());
  }
  if (nextVisibility?.trim() && nextVisibility !== "all") {
    params.set("visibility", nextVisibility.trim());
  }
  if (nextSort?.trim() && nextSort !== "downloads-desc") {
    params.set("sort", nextSort.trim());
  }
  if (nextArtifactTab?.trim() && nextArtifactTab !== "packages") {
    params.set("artifactTab", nextArtifactTab.trim());
  }
  if (nextPage && String(nextPage) !== "1") {
    params.set("page", String(nextPage));
  }
  if (nextPageSize && String(nextPageSize) !== "30") {
    params.set("pageSize", String(nextPageSize));
  }

  const base =
    ownerKind === "organization"
      ? `${organizationHref(owner)}/packages`
      : `/${encodeURIComponent(owner)}`;
  const query = params.toString();
  return `${base}${query ? `?${query}` : ""}`;
}

export function packageDetailHref(
  ownerKind: "user" | "organization",
  owner: string,
  packageType: string,
  packageName: string,
  version?: string | null,
) {
  const base =
    ownerKind === "organization"
      ? `${organizationHref(owner)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}`
      : `/${encodeURIComponent(owner)}/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}`;
  return version?.trim()
    ? `${base}?version=${encodeURIComponent(version.trim())}`
    : base;
}

export function organizationProjectHref(org: string) {
  return `${organizationHref(org)}/projects`;
}

export function organizationSettingsHref(org: string) {
  return `${organizationHref(org)}/settings`;
}

export function organizationSettingsSectionHref(
  org: string,
  item: OrganizationSettingsSection,
) {
  return `/organizations/${encodeURIComponent(org)}${item.href}`;
}

export function organizationTeamHref(org: string, teamSlug: string) {
  return `${organizationHref(org)}/teams/${encodeURIComponent(teamSlug)}`;
}

export function activeSearchType(value: string | null | undefined) {
  return tabValue(SEARCH_TABS, value ?? null);
}

export function searchTypeHref(type: string, query: string | null | undefined) {
  return queryTabHref("/search", "type", type, { q: query });
}

export function codeSearchHref(
  query: string,
  extraParams: Record<string, string | null | undefined> = {},
) {
  return searchHref(query, "code", extraParams);
}

export function quoteSearchQualifierValue(value: string) {
  const trimmed = value.trim();
  return /\s/.test(trimmed) ? `"${trimmed.replaceAll('"', '\\"')}"` : trimmed;
}

function searchQualifierTokenPattern(qualifier: string, value?: string) {
  const escapedQualifier = qualifier.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  if (!value) {
    return new RegExp(`(^|\\s)${escapedQualifier}:(?:"[^"]*"|\\S+)`, "gi");
  }
  const escapedValue = quoteSearchQualifierValue(value).replace(
    /[.*+?^${}()|[\]\\]/g,
    "\\$&",
  );
  return new RegExp(
    `(^|\\s)${escapedQualifier}:${escapedValue}(?=\\s|$)`,
    "gi",
  );
}

export function removeCodeSearchQualifier(
  query: string,
  qualifier: string,
  value?: string,
) {
  return query
    .replace(searchQualifierTokenPattern(qualifier, value), " ")
    .split(/\s+/)
    .filter(Boolean)
    .join(" ");
}

export function addCodeSearchQualifierHref(
  query: string,
  qualifier: string,
  value: string,
) {
  const baseQuery = removeCodeSearchQualifier(query, qualifier, value);
  const nextQuery =
    `${baseQuery.trim()} ${qualifier}:${quoteSearchQualifierValue(value)}`.trim();
  return codeSearchHref(nextQuery);
}

export function toggleCodeSearchQualifierHref(
  query: string,
  qualifier: string,
  value: string,
  selected: boolean,
) {
  if (selected) {
    return codeSearchHref(removeCodeSearchQualifier(query, qualifier, value));
  }
  return addCodeSearchQualifierHref(query, qualifier, value);
}

export function removeCodeSearchQualifierHref(removeQuery: string) {
  return codeSearchHref(removeQuery);
}

export function codeSearchViewHref(query: string, view: string) {
  return codeSearchHref(query, { view });
}

export function searchQueryHref(query: string) {
  return searchTypeHref("repositories", query);
}

export function repositoryJumpHref(owner: string, repo: string) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`;
}

export function profileHref(owner: string) {
  return `/${encodeURIComponent(owner)}`;
}

export type ProfileRepositoryTabFilters = {
  query?: string | null;
  repositoryType?: string | null;
  language?: string | null;
  sort?: string | null;
  mode?: string | null;
};

export function profileRepositoryTabHref(
  owner: string,
  filters: ProfileRepositoryTabFilters = {},
  overrides: Partial<
    Record<"q" | "type" | "language" | "sort", string | null>
  > = {},
) {
  const params = new URLSearchParams();
  const tab = filters.mode === "stars" ? "stars" : "repositories";
  const defaultSort = tab === "stars" ? "recently-starred" : "updated-desc";
  params.set("tab", tab);

  const nextQuery =
    overrides.q === undefined ? filters.query : overrides.q?.trim() || null;
  const nextType =
    overrides.type === undefined
      ? filters.repositoryType
      : overrides.type?.trim() || "all";
  const nextLanguage =
    overrides.language === undefined
      ? filters.language
      : overrides.language?.trim() || null;
  const nextSort =
    overrides.sort === undefined ? filters.sort : overrides.sort?.trim() || "";

  if (nextQuery?.trim()) {
    params.set("q", nextQuery.trim());
  }
  if (nextType?.trim() && nextType !== "all") {
    params.set("type", nextType.trim());
  }
  if (nextLanguage?.trim()) {
    params.set("language", nextLanguage.trim());
  }
  if (nextSort?.trim() && nextSort !== defaultSort) {
    params.set("sort", nextSort.trim());
  }

  return `${profileHref(owner)}?${params.toString()}`;
}

export function createJumpSuggestions(): JumpSuggestion[] {
  return CREATE_NAV_ITEMS.map((item) => ({
    id: `create:${item.href}`,
    kind: "create",
    label: item.label,
    description: item.description,
    href: item.href,
    section: "Create",
  }));
}

export function queryJumpSuggestions(query: string): JumpSuggestion[] {
  const normalized = query.trim();
  if (!normalized) {
    return [];
  }

  return [
    {
      id: `search:${normalized}`,
      kind: "search",
      label: `Search repositories for "${normalized}"`,
      description: "Press Enter",
      href: searchQueryHref(normalized),
      section: "Search",
    },
  ];
}

export function isActivePath(pathname: string, href: string): boolean {
  if (href === "/") {
    return pathname === "/";
  }

  return pathname === href || pathname.startsWith(`${href}/`);
}

export function activeSettingsSection(pathname: string): string {
  return (
    SETTINGS_NAV_ITEMS.find((item) => isActivePath(pathname, item.href))
      ?.section ?? "profile"
  );
}

export function repositorySettingsHref(
  owner: string,
  repo: string,
  item: RepositorySettingsSection,
) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}${item.hrefSuffix}`;
}

export function activeRepositorySettingsSection(pathname: string): string {
  const [, owner, repo, settings, section] = pathname.split("/");

  if (!owner || !repo || settings !== "settings") {
    return "general";
  }

  return (
    REPOSITORY_SETTINGS_NAV_ITEMS.find((item) => item.section === section)
      ?.section ?? "general"
  );
}

export function repositoryTabHref(
  owner: string,
  repo: string,
  tab: RepositoryTab,
) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}${tab.href}`;
}

export type RepositoryIssueHrefQuery = {
  q?: string | null;
  state?: string | null;
  author?: string | null;
  excludedAuthor?: string | null;
  labels?: string[] | null;
  excludedLabels?: string[] | null;
  noLabels?: boolean | null;
  milestone?: string | null;
  noMilestone?: boolean | null;
  assignee?: string | null;
  noAssignee?: boolean | null;
  project?: string | null;
  issueType?: string | null;
  sort?: string | null;
  page?: number | null;
};

export function repositoryIssuesHref(
  owner: string,
  repo: string,
  query: RepositoryIssueHrefQuery = {},
) {
  const params = new URLSearchParams();
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state?.trim()) {
    params.set("state", query.state.trim());
  }
  if (query.author?.trim()) {
    params.set("author", query.author.trim());
  }
  if (query.excludedAuthor?.trim()) {
    params.set("excludedAuthor", query.excludedAuthor.trim());
  }
  if (query.labels?.length) {
    params.set("labels", query.labels.join(","));
  }
  if (query.excludedLabels?.length) {
    params.set("excludedLabels", query.excludedLabels.join(","));
  }
  if (query.noLabels) {
    params.set("noLabels", "true");
  }
  if (query.milestone?.trim()) {
    params.set("milestone", query.milestone.trim());
  }
  if (query.noMilestone) {
    params.set("noMilestone", "true");
  }
  if (query.assignee?.trim()) {
    params.set("assignee", query.assignee.trim());
  }
  if (query.noAssignee) {
    params.set("noAssignee", "true");
  }
  if (query.project?.trim()) {
    params.set("project", query.project.trim());
  }
  if (query.issueType?.trim()) {
    params.set("issueType", query.issueType.trim());
  }
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }

  const suffix = params.size ? `?${params.toString()}` : "";
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues${suffix}`;
}

export function repositoryIssueDetailHref(
  owner: string,
  repo: string,
  issueNumber: number,
) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${issueNumber}`;
}

export type RepositoryPullRequestHrefQuery = {
  q?: string | null;
  state?: "open" | "closed" | "merged" | null;
  author?: string | null;
  labels?: string[] | null;
  milestone?: string | null;
  noMilestone?: boolean | null;
  assignee?: string | null;
  noAssignee?: boolean | null;
  project?: string | null;
  review?: string | null;
  checks?: string | null;
  sort?: string | null;
  page?: number | null;
};

export function repositoryPullRequestsHref(
  owner: string,
  repo: string,
  query: RepositoryPullRequestHrefQuery = {},
) {
  const params = new URLSearchParams();
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state) {
    params.set("state", query.state);
  }
  if (query.author?.trim()) {
    params.set("author", query.author.trim());
  }
  if (query.labels?.length) {
    params.set("labels", query.labels.join(","));
  }
  if (query.milestone?.trim()) {
    params.set("milestone", query.milestone.trim());
  }
  if (query.noMilestone) {
    params.set("noMilestone", "true");
  }
  if (query.assignee?.trim()) {
    params.set("assignee", query.assignee.trim());
  }
  if (query.noAssignee) {
    params.set("noAssignee", "true");
  }
  if (query.project?.trim()) {
    params.set("project", query.project.trim());
  }
  if (query.review?.trim()) {
    params.set("review", query.review.trim());
  }
  if (query.checks?.trim()) {
    params.set("checks", query.checks.trim());
  }
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls${suffix}`;
}

export function repositoryPullRequestDetailHref(
  owner: string,
  repo: string,
  pullNumber: number,
) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pull/${pullNumber}`;
}

export function repositoryPullRequestCompareHref(owner: string, repo: string) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/compare`;
}

export type RepositoryCompareRange = {
  base: string;
  head: string;
};

export function parseRepositoryCompareRange(
  range: string | string[] | null | undefined,
): RepositoryCompareRange | null {
  const raw = Array.isArray(range) ? range.join("/") : range;
  const decoded = decodeURIComponent(raw ?? "").trim();
  const separator = decoded.indexOf("...");
  if (separator <= 0 || separator === decoded.length - 3) {
    return null;
  }
  const base = decoded.slice(0, separator).trim();
  const head = decoded.slice(separator + 3).trim();
  if (!base || !head) {
    return null;
  }
  return { base, head };
}

export function repositoryCompareRangeHref(
  owner: string,
  repo: string,
  base: string,
  head: string,
  query: {
    view?: "split" | "unified";
    headOwner?: string | null;
    headRepo?: string | null;
  } = {},
) {
  const params = new URLSearchParams();
  if (query.view && query.view !== "split") {
    params.set("view", query.view);
  }
  if (query.headOwner && query.headRepo) {
    params.set("headOwner", query.headOwner);
    params.set("headRepo", query.headRepo);
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head)}${suffix}`;
}

export function repositoryCompareSwapHref(
  owner: string,
  repo: string,
  base: string,
  head: string,
  query: {
    view?: "split" | "unified";
    headOwner?: string | null;
    headRepo?: string | null;
  } = {},
) {
  return repositoryCompareRangeHref(owner, repo, head, base, query);
}

export function repositoryCompareViewHref(
  owner: string,
  repo: string,
  base: string,
  head: string,
  view: "split" | "unified",
  query: { headOwner?: string | null; headRepo?: string | null } = {},
) {
  return repositoryCompareRangeHref(owner, repo, base, head, {
    view,
    headOwner: query.headOwner,
    headRepo: query.headRepo,
  });
}

export function repositoryPullRequestStateHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  state: "open" | "closed" | "merged",
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    state,
    q: pullRequestQueryWithState(current.q, state),
    page: null,
  });
}

export function repositoryPullRequestPageHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  page: number,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    page,
  });
}

export function repositoryPullRequestSortHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  sort: string,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    sort,
    q: removePullRequestFilterFromQuery(
      removePullRequestFilterFromQuery(current.q, "sort"),
      "order",
    ),
    page: null,
  });
}

export function repositoryPullRequestSetLabelHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  label: string,
) {
  const labels = uniqueIssueValues([...(current.labels ?? []), label]);
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    labels,
    q: addPullRequestQualifier(
      removePullRequestFilterFromQuery(current.q, "label", label),
      "label",
      label,
    ),
    page: null,
  });
}

export function repositoryPullRequestSetUserFilterHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  filter: "author" | "assignee",
  login: string,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    [filter]: login,
    ...(filter === "assignee" ? { noAssignee: false } : {}),
    q: addPullRequestQualifier(
      filter === "assignee"
        ? removeNoIssueFilterFromQuery(
            removePullRequestFilterFromQuery(current.q, "assignee"),
            "assignee",
          )
        : removePullRequestFilterFromQuery(current.q, "author"),
      filter,
      login,
    ),
    page: null,
  });
}

export function repositoryPullRequestNoAssigneeHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    assignee: null,
    noAssignee: true,
    q: addPullRequestQualifier(
      removePullRequestFilterFromQuery(
        removeNoIssueFilterFromQuery(current.q, "assignee"),
        "assignee",
      ),
      "no",
      "assignee",
    ),
    page: null,
  });
}

export function repositoryPullRequestSetMilestoneHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  milestone: string,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    milestone,
    noMilestone: false,
    q: addPullRequestQualifier(
      removeNoIssueFilterFromQuery(
        removePullRequestFilterFromQuery(current.q, "milestone"),
        "milestone",
      ),
      "milestone",
      milestone,
    ),
    page: null,
  });
}

export function repositoryPullRequestNoMilestoneHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    milestone: null,
    noMilestone: true,
    q: addPullRequestQualifier(
      removePullRequestFilterFromQuery(
        removeNoIssueFilterFromQuery(current.q, "milestone"),
        "milestone",
      ),
      "no",
      "milestone",
    ),
    page: null,
  });
}

export function repositoryPullRequestSetReviewHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  review: string,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    review,
    q: addPullRequestQualifier(
      removePullRequestFilterFromQuery(current.q, "review"),
      "review",
      review,
    ),
    page: null,
  });
}

export function repositoryPullRequestSetChecksHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  checks: string,
) {
  return repositoryPullRequestsHref(owner, repo, {
    ...current,
    checks,
    q: addPullRequestQualifier(
      removePullRequestFilterFromQuery(current.q, "checks"),
      "checks",
      checks,
    ),
    page: null,
  });
}

export function repositoryPullRequestClearFilterHref(
  owner: string,
  repo: string,
  current: RepositoryPullRequestHrefQuery,
  filter:
    | "author"
    | "labels"
    | "milestone"
    | "noMilestone"
    | "assignee"
    | "noAssignee"
    | "project"
    | "review"
    | "checks",
  value?: string,
) {
  const next = { ...current, page: null };
  if (filter === "author") {
    next.author = null;
    next.q = removePullRequestFilterFromQuery(current.q, "author");
  } else if (filter === "labels") {
    next.labels = (current.labels ?? []).filter(
      (label) => label.toLowerCase() !== value?.toLowerCase(),
    );
    next.q = removePullRequestFilterFromQuery(current.q, "label", value);
  } else if (filter === "assignee") {
    next.assignee = null;
    next.q = removePullRequestFilterFromQuery(current.q, "assignee");
  } else if (filter === "noAssignee") {
    next.noAssignee = false;
    next.q = removeNoIssueFilterFromQuery(current.q, "assignee");
  } else if (filter === "noMilestone") {
    next.noMilestone = false;
    next.q = removeNoIssueFilterFromQuery(current.q, "milestone");
  } else if (filter === "project") {
    next.project = null;
    next.q = removePullRequestFilterFromQuery(current.q, "project");
  } else {
    next[filter] = null;
    next.q = removePullRequestFilterFromQuery(current.q, filter);
  }
  return repositoryPullRequestsHref(owner, repo, next);
}

export function repositoryIssueStateHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  state: "open" | "closed",
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    state,
    q: issueQueryWithState(current.q, state),
    page: null,
  });
}

export function repositoryIssueSortHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  sort: string,
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    sort,
    q: removeIssueFilterFromQuery(
      removeIssueFilterFromQuery(current.q, "sort"),
      "order",
    ),
    page: null,
  });
}

export function repositoryIssueClearFilterHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  filter:
    | "author"
    | "excludedAuthor"
    | "labels"
    | "excludedLabels"
    | "noLabels"
    | "milestone"
    | "noMilestone"
    | "assignee"
    | "noAssignee"
    | "project"
    | "issueType",
  value?: string,
) {
  const next = { ...current, page: null };
  if (filter === "author") {
    next.author = null;
    next.q = removeIssueFilterFromQuery(current.q, "author");
  } else if (filter === "excludedAuthor") {
    next.excludedAuthor = null;
    next.q = removeIssueFilterFromQuery(current.q, "-author");
  } else if (filter === "labels") {
    next.labels = value
      ? current.labels?.filter(
          (label) => label.toLowerCase() !== value.toLowerCase(),
        )
      : [];
    next.q = removeIssueFilterFromQuery(current.q, "label", value);
  } else if (filter === "excludedLabels") {
    next.excludedLabels = value
      ? current.excludedLabels?.filter(
          (label) => label.toLowerCase() !== value.toLowerCase(),
        )
      : [];
    next.q = removeIssueFilterFromQuery(current.q, "-label", value);
  } else if (filter === "noLabels") {
    next.noLabels = false;
    next.q = removeNoLabelFilterFromQuery(current.q);
  } else if (filter === "milestone") {
    next.milestone = null;
    next.q = removeIssueFilterFromQuery(current.q, "milestone");
  } else if (filter === "noMilestone") {
    next.noMilestone = false;
    next.q = removeNoIssueFilterFromQuery(current.q, "milestone");
  } else if (filter === "assignee") {
    next.assignee = null;
    next.q = removeIssueFilterFromQuery(current.q, "assignee");
  } else if (filter === "noAssignee") {
    next.noAssignee = false;
    next.q = removeNoIssueFilterFromQuery(current.q, "assignee");
  } else if (filter === "project") {
    next.project = null;
    next.q = removeIssueFilterFromQuery(current.q, "project");
  } else {
    next.issueType = null;
    next.q = removeIssueFilterFromQuery(current.q, "type");
  }
  return repositoryIssuesHref(owner, repo, next);
}

export function repositoryIssueSetUserFilterHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  filter: "author" | "assignee",
  login: string,
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    [filter]: login,
    ...(filter === "author" ? { excludedAuthor: null } : { noAssignee: false }),
    q: addIssueQualifier(
      filter === "author"
        ? removeIssueFilterFromQuery(
            removeIssueFilterFromQuery(current.q, "-author"),
            "author",
          )
        : removeNoIssueFilterFromQuery(
            removeIssueFilterFromQuery(current.q, "assignee"),
            "assignee",
          ),
      filter,
      login,
    ),
    page: null,
  });
}

export function repositoryIssueExcludeAuthorHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  login: string,
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    author: null,
    excludedAuthor: login,
    q: addIssueQualifier(
      removeIssueFilterFromQuery(
        removeIssueFilterFromQuery(current.q, "author"),
        "-author",
      ),
      "-author",
      login,
    ),
    page: null,
  });
}

export function repositoryIssueSetMilestoneHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  milestone: string,
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    milestone,
    noMilestone: false,
    q: addIssueQualifier(
      removeNoIssueFilterFromQuery(
        removeIssueFilterFromQuery(current.q, "milestone"),
        "milestone",
      ),
      "milestone",
      milestone,
    ),
    page: null,
  });
}

export function repositoryIssueNoMetadataHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  filter: "assignee" | "milestone",
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    ...(filter === "assignee"
      ? { assignee: null, noAssignee: true }
      : { milestone: null, noMilestone: true }),
    q: addIssueQualifier(
      removeIssueFilterFromQuery(
        removeNoIssueFilterFromQuery(current.q, filter),
        filter,
      ),
      "no",
      filter,
    ),
    page: null,
  });
}

export function repositoryIssueAddLabelHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  label: string,
) {
  const labels = uniqueIssueValues([...(current.labels ?? []), label]);
  const excludedLabels = (current.excludedLabels ?? []).filter(
    (value) => value.toLowerCase() !== label.toLowerCase(),
  );
  return repositoryIssuesHref(owner, repo, {
    ...current,
    labels,
    excludedLabels,
    noLabels: false,
    q: addIssueQualifier(
      removeIssueFilterFromQuery(
        removeNoLabelFilterFromQuery(current.q),
        "-label",
        label,
      ),
      "label",
      label,
    ),
    page: null,
  });
}

export function repositoryIssueExcludeLabelHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
  label: string,
) {
  const excludedLabels = uniqueIssueValues([
    ...(current.excludedLabels ?? []),
    label,
  ]);
  const labels = (current.labels ?? []).filter(
    (value) => value.toLowerCase() !== label.toLowerCase(),
  );
  return repositoryIssuesHref(owner, repo, {
    ...current,
    labels,
    excludedLabels,
    noLabels: false,
    q: addIssueQualifier(
      removeIssueFilterFromQuery(
        removeNoLabelFilterFromQuery(current.q),
        "label",
        label,
      ),
      "-label",
      label,
    ),
    page: null,
  });
}

export function repositoryIssueNoLabelsHref(
  owner: string,
  repo: string,
  current: RepositoryIssueHrefQuery,
) {
  return repositoryIssuesHref(owner, repo, {
    ...current,
    labels: [],
    excludedLabels: [],
    noLabels: true,
    q: addIssueQualifier(
      removeNoLabelFilterFromQuery(
        removeIssueFilterFromQuery(
          removeIssueFilterFromQuery(current.q, "label"),
          "-label",
        ),
      ),
      "no",
      "label",
    ),
    page: null,
  });
}

function issueQueryWithState(
  query: string | null | undefined,
  state: "open" | "closed",
) {
  const terms = (query?.trim() || "is:issue")
    .split(/\s+/)
    .filter(
      (term) =>
        term &&
        !term.startsWith("state:") &&
        term !== "is:open" &&
        term !== "is:closed",
    );
  if (!terms.some((term) => term === "is:issue")) {
    terms.unshift("is:issue");
  }
  terms.push(`state:${state}`);
  return terms.join(" ");
}

function pullRequestQueryWithState(
  query: string | null | undefined,
  state: "open" | "closed" | "merged",
) {
  const terms = (query?.trim() || "is:pr")
    .split(/\s+/)
    .filter(
      (term) =>
        term &&
        !term.startsWith("state:") &&
        term !== "is:open" &&
        term !== "is:closed" &&
        term !== "is:merged",
    );
  if (!terms.some((term) => term === "is:pr")) {
    terms.unshift("is:pr");
  }
  terms.push(`state:${state}`);
  return terms.join(" ");
}

function removePullRequestFilterFromQuery(
  query: string | null | undefined,
  filter:
    | "author"
    | "label"
    | "milestone"
    | "project"
    | "assignee"
    | "review"
    | "checks"
    | "sort"
    | "order",
  value?: string,
) {
  const normalizedValue = value?.toLowerCase();
  return issueQueryTerms(query?.trim() || "")
    .filter((term) => {
      const prefix = `${filter}:`;
      if (!term.startsWith(prefix)) {
        return true;
      }
      if (!normalizedValue) {
        return false;
      }
      return (
        term.slice(prefix.length).replaceAll('"', "").toLowerCase() !==
        normalizedValue
      );
    })
    .join(" ");
}

function addPullRequestQualifier(
  query: string | null | undefined,
  filter:
    | "author"
    | "label"
    | "milestone"
    | "assignee"
    | "review"
    | "checks"
    | "no",
  value: string,
) {
  return addIssueQualifier(query, filter, value);
}

function removeIssueFilterFromQuery(
  query: string | null | undefined,
  filter:
    | "author"
    | "-author"
    | "label"
    | "-label"
    | "milestone"
    | "assignee"
    | "project"
    | "type"
    | "sort"
    | "order",
  value?: string,
) {
  const normalizedValue = value?.toLowerCase();
  return issueQueryTerms(query?.trim() || "")
    .filter((term) => {
      const prefix = `${filter}:`;
      if (!term.startsWith(prefix)) {
        return true;
      }
      if (!normalizedValue) {
        return false;
      }
      return (
        term.slice(prefix.length).replaceAll('"', "").toLowerCase() !==
        normalizedValue
      );
    })
    .join(" ");
}

function removeNoLabelFilterFromQuery(query: string | null | undefined) {
  return removeNoIssueFilterFromQuery(
    removeNoIssueFilterFromQuery(query, "labels"),
    "label",
  );
}

function removeNoIssueFilterFromQuery(
  query: string | null | undefined,
  value: "label" | "labels" | "assignee" | "milestone",
) {
  const target = `no:${value}`;
  return issueQueryTerms(query?.trim() || "")
    .filter((term) => term !== target)
    .join(" ");
}

function addIssueQualifier(
  query: string | null | undefined,
  filter:
    | "author"
    | "-author"
    | "label"
    | "-label"
    | "milestone"
    | "assignee"
    | "project"
    | "type"
    | "review"
    | "checks"
    | "sort"
    | "order"
    | "no",
  value: string,
) {
  const terms = issueQueryTerms(query?.trim() || "");
  const normalized = value.trim();
  if (!normalized) {
    return terms.join(" ");
  }
  const next = `${filter}:${quoteIssueQualifierValue(normalized)}`;
  if (
    !terms.some(
      (term) => term.replaceAll('"', "").toLowerCase() === next.toLowerCase(),
    )
  ) {
    terms.push(next);
  }
  return terms.join(" ").trim();
}

function quoteIssueQualifierValue(value: string) {
  return /\s/.test(value) ? `"${value.replaceAll('"', '\\"')}"` : value;
}

function uniqueIssueValues(values: string[]) {
  const seen = new Set<string>();
  return values.filter((value) => {
    const key = value.toLowerCase();
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function issueQueryTerms(query: string) {
  const terms: string[] = [];
  let rest = query.trim();
  while (rest) {
    const spaceIndex = rest.search(/\s/);
    const tokenEnd = spaceIndex === -1 ? rest.length : spaceIndex;
    const token = rest.slice(0, tokenEnd);
    const quoteIndex = token.indexOf(':"');
    if (quoteIndex >= 0) {
      const prefixLength = quoteIndex + 2;
      const quotedRest = rest.slice(prefixLength);
      const endQuote = quotedRest.indexOf('"');
      if (endQuote >= 0) {
        terms.push(
          `${token.slice(0, prefixLength)}${quotedRest.slice(0, endQuote + 1)}`,
        );
        rest = quotedRest.slice(endQuote + 1).trimStart();
      } else {
        terms.push(rest);
        rest = "";
      }
    } else {
      terms.push(token);
      rest = rest.slice(tokenEnd).trimStart();
    }
  }
  return terms;
}

export function activeRepositoryTab(pathname: string): string {
  const [, , , segment = ""] = pathname.split("/");

  if (segment === "pull") {
    return "pulls";
  }

  if (segment === "graphs" || segment === "network" || segment === "forks") {
    return "pulse";
  }

  return REPOSITORY_TABS.some((tab) => tab.segment === segment) ? segment : "";
}
