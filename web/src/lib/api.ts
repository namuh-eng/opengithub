export type AuthUser = {
  id: string;
  email: string;
  display_name: string | null;
  avatar_url: string | null;
};

export type AuthSession = {
  authenticated: boolean;
  user: AuthUser | null;
};

export type RepositoryVisibility = "public" | "private" | "internal";

export type PublicUserProfile = {
  identity: ProfileIdentity;
  readme: ProfileReadme | null;
  pinnedRepositories: ProfilePinnedRepository[];
  achievements: ProfileAchievement[];
  organizations: ProfileOrganization[];
  contributionSummary: ProfileContributionSummary;
  tabCounts: ProfileTabCounts;
  viewerState: ProfileViewerState;
};

export type ProfileIdentity = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  bio: string | null;
  company: string | null;
  location: string | null;
  websiteUrl: string | null;
  htmlUrl: string;
  profileVisibility: "public" | "private" | string;
  isPrivate: boolean;
  joinedAt: string;
  followerCount: number | null;
  followingCount: number | null;
};

export type ProfileReadme = {
  body: string;
  renderedHtml: string | null;
  updatedAt: string;
};

export type ProfilePinnedRepository = {
  id: string;
  owner: string;
  name: string;
  description: string | null;
  visibility: RepositoryVisibility;
  href: string;
  defaultBranch: string;
  primaryLanguage: ProfileRepositoryLanguage | null;
  languages: ProfileRepositoryLanguage[];
  starsCount: number;
  forksCount: number;
  updatedAt: string;
};

export type ProfileRepositoryList = {
  items: ProfileRepositoryListItem[];
  total: number;
  page: number;
  pageSize: number;
  mode: "repositories" | "stars" | string;
  filters: ProfileRepositoryFilters;
  availableLanguages: ProfileRepositoryFilterOption[];
  availableTypes: ProfileRepositoryFilterOption[];
  tabCounts: ProfileTabCounts;
};

export type ProfileRepositoryListItem = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  description: string | null;
  visibility: RepositoryVisibility;
  href: string;
  defaultBranch: string;
  primaryLanguage: ProfileRepositoryLanguage | null;
  languages: ProfileRepositoryLanguage[];
  starsCount: number;
  forksCount: number;
  openIssuesCount: number;
  openPullRequestsCount: number;
  license: ProfileRepositoryLicense | null;
  isArchived: boolean;
  isFork: boolean;
  isTemplate: boolean;
  isMirror: boolean;
  canBeSponsored: boolean;
  forkSource: ProfileRepositoryForkSource | null;
  starredAt?: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ProfileRepositoryLicense = {
  slug: string;
  name: string;
};

export type ProfileRepositoryForkSource = {
  owner: string;
  name: string;
  href: string;
};

export type ProfileRepositoryFilters = {
  query: string | null;
  repositoryType: string;
  language: string | null;
  sort: string;
  page: number;
  pageSize: number;
};

export type ProfileRepositoryFilterOption = {
  value: string;
  label: string;
  count: number;
};

export type ProfileRepositoryListQuery = {
  q?: string;
  type?: string;
  language?: string;
  sort?: string;
  page?: number;
  pageSize?: number;
};

export type ProfileRepositoryLanguage = {
  language: string;
  color: string;
  byteCount: number;
};

export type PublicOrganizationProfile = {
  identity: OrganizationIdentity;
  verifiedDomains: OrganizationVerifiedDomain[];
  pinnedRepositories: OrganizationRepositoryPreview[];
  repositoryPreview: OrganizationRepositoryPreview[];
  peoplePreview: OrganizationPersonPreview[];
  topLanguages: OrganizationLanguageSummary[];
  topTopics: OrganizationTopicSummary[];
  sponsorship: OrganizationSponsorshipState;
  tabCounts: OrganizationTabCounts;
  viewerState: OrganizationViewerState;
};

export type OrganizationIdentity = {
  id: string;
  slug: string;
  name: string;
  description: string | null;
  avatarUrl: string | null;
  websiteUrl: string | null;
  location: string | null;
  htmlUrl: string;
  profileVisibility: "public" | "private" | string;
  isPrivate: boolean;
  followerCount: number;
  publicMemberCount: number;
  repositoryCount: number;
  createdAt: string;
};

export type OrganizationVerifiedDomain = {
  domain: string;
  verifiedAt: string;
  href: string;
};

export type OrganizationRepositoryPreview = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  description: string | null;
  visibility: RepositoryVisibility;
  href: string;
  defaultBranch: string;
  primaryLanguage: OrganizationLanguageSummary | null;
  languages: OrganizationLanguageSummary[];
  topics: string[];
  starsCount: number;
  forksCount: number;
  openIssuesCount: number;
  openPullRequestsCount: number;
  isArchived: boolean;
  isTemplate: boolean;
  isMirror: boolean;
  license: OrganizationRepositoryLicense | null;
  updatedAt: string;
};

export type OrganizationRepositoryLicense = {
  slug: string;
  name: string;
};

export type OrganizationPersonPreview = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  href: string;
  role: string | null;
};

export type OrganizationLanguageSummary = {
  language: string;
  color: string;
  byteCount: number;
};

export type OrganizationTopicSummary = {
  topic: string;
  count: number;
  href: string;
};

export type OrganizationSponsorshipState = {
  enabled: boolean;
  sponsorCount: number;
  href: string | null;
  unavailableReason: string | null;
};

export type OrganizationTabCounts = {
  repositories: number;
  projects: number;
  packages: number;
  people: number;
  sponsoring: number;
};

export type OrganizationViewerState = {
  authenticated: boolean;
  isMember: boolean;
  role: string | null;
  canViewInternal: boolean;
  canAdmin: boolean;
  isFollowing: boolean;
};

export type OrganizationRepositoryList = {
  items: OrganizationRepositoryListItem[];
  total: number;
  page: number;
  pageSize: number;
  mode: "repositories" | string;
  filters: OrganizationRepositoryFilters;
  availableLanguages: OrganizationRepositoryFilterOption[];
  availableTypes: OrganizationRepositoryFilterOption[];
  tabCounts: OrganizationTabCounts;
  viewerState: OrganizationViewerState;
};

export type OrganizationRepositoryListItem = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  description: string | null;
  visibility: RepositoryVisibility;
  href: string;
  defaultBranch: string;
  primaryLanguage: OrganizationLanguageSummary | null;
  languages: OrganizationLanguageSummary[];
  topics: string[];
  starsCount: number;
  forksCount: number;
  openIssuesCount: number;
  openPullRequestsCount: number;
  license: OrganizationRepositoryLicense | null;
  isArchived: boolean;
  isFork: boolean;
  isTemplate: boolean;
  isMirror: boolean;
  canAdmin: boolean;
  contributedByViewer: boolean;
  forkSource: OrganizationRepositoryForkSource | null;
  createdAt: string;
  updatedAt: string;
};

export type OrganizationRepositoryForkSource = {
  owner: string;
  name: string;
  href: string;
};

export type OrganizationRepositoryFilters = {
  query: string | null;
  repositoryType: string;
  language: string | null;
  sort: string;
  density: string;
  page: number;
  pageSize: number;
};

export type OrganizationRepositoryFilterOption = {
  value: string;
  label: string;
  count: number;
};

export type OrganizationRepositoryListQuery = {
  q?: string;
  type?: string;
  language?: string;
  sort?: string;
  density?: string;
  page?: number;
  pageSize?: number;
};

export type OrganizationPeopleList = {
  items: OrganizationPeopleListItem[];
  total: number;
  page: number;
  pageSize: number;
  mode: "people" | string;
  filters: OrganizationPeopleFilters;
  tabCounts: OrganizationTabCounts;
  viewerState: OrganizationViewerState;
};

export type OrganizationPeopleListItem = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  href: string;
  role: string | null;
  joinedAt: string;
};

export type OrganizationPeopleFilters = {
  query: string | null;
  page: number;
  pageSize: number;
};

export type OrganizationPeopleListQuery = {
  q?: string;
  page?: number;
  pageSize?: number;
};

export type ProfileAchievement = {
  slug: string;
  name: string;
  description: string | null;
  icon: string | null;
  awardedAt: string;
};

export type ProfileOrganization = {
  id: string;
  slug: string;
  name: string;
  avatarUrl: string | null;
  href: string;
};

export type ProfileContributionSummary = {
  total: number;
  year: number;
  days: ProfileContributionDay[];
  recentEvents: ProfileContributionEvent[];
};

export type ProfileContributionDay = {
  date: string;
  count: number;
  intensity: number;
};

export type ProfileContributionEvent = {
  id: string;
  eventType: string;
  title: string;
  targetHref: string | null;
  occurredAt: string;
  repository: ProfileEventRepository | null;
};

export type ProfileEventRepository = {
  owner: string;
  name: string;
  href: string;
};

export type ProfileTabCounts = {
  repositories: number;
  projects: number;
  packages: number;
  stars: number;
};

export type ProfileViewerState = {
  authenticated: boolean;
  isSelf: boolean;
  isFollowing: boolean;
  isBlocking: boolean;
  canFollow: boolean;
  canBlock: boolean;
  canReport: boolean;
};

export type ProfileActionState = {
  viewerState: ProfileViewerState;
  followerCount: number | null;
};

export type ProfileReport = {
  id: string;
  viewerState: ProfileViewerState;
};

export type ReportUserRequest = {
  reason: string;
  details?: string;
};

export type SearchSuggestionToken = {
  prefix: string | null;
  value: string;
  replaceFrom: number;
  replaceTo: number;
};

export type SearchSuggestionItem = {
  id: string;
  kind: string;
  action:
    | "navigate"
    | "submit_search"
    | "replace_token"
    | "open_saved_search_dialog";
  title: string;
  description: string | null;
  href: string | null;
  nextQuery: string | null;
  scope: string | null;
  ownerLogin: string | null;
  repositoryName: string | null;
  visibility: RepositoryVisibility | null;
};

export type SearchSuggestionGroup = {
  id: string;
  title: string;
  items: SearchSuggestionItem[];
};

export type SavedSearchSuggestion = {
  id: string;
  name: string;
  query: string;
  scope: string;
  href: string;
  updatedAt: string;
};

export type RecentSearchSuggestion = {
  id: string;
  query: string;
  scope: string;
  resultType: string | null;
  href: string;
  searchedAt: string;
};

export type SearchSuggestionDashboard = {
  query: string;
  scope: string;
  token: SearchSuggestionToken | null;
  groups: SearchSuggestionGroup[];
  savedSearches: SavedSearchSuggestion[];
  recentSearches: RecentSearchSuggestion[];
};

export type SearchSuggestionsQuery = {
  query?: string;
  scope?: string;
  limit?: number;
};

export type CreateSavedSearchRequest = {
  name: string;
  query: string;
  scope?: string;
};

export type RepositoryOwnerType = "user" | "organization";

export type RepositorySummary = {
  id: string;
  owner_user_id: string | null;
  owner_organization_id: string | null;
  owner_login: string;
  name: string;
  description: string | null;
  visibility: RepositoryVisibility;
  default_branch: string;
  is_archived: boolean;
  created_by_user_id: string;
  created_at: string;
  updated_at: string;
};

export type RepositoryFile = {
  id: string;
  repositoryId: string;
  commitId: string;
  path: string;
  content: string;
  oid: string;
  byteSize: number;
  createdAt: string;
};

export type RepositoryTreeEntry = {
  kind: "folder" | "file" | string;
  name: string;
  path: string;
  href: string;
  byteSize: number | null;
  latestCommitMessage: string | null;
  latestCommitHref: string | null;
  updatedAt: string;
};

export type RepositoryPathBreadcrumb = {
  name: string;
  path: string;
  href: string;
};

export type RepositoryLatestCommit = {
  oid: string;
  shortOid: string;
  message: string;
  href: string;
  committedAt: string;
};

export type RepositoryResolvedRef = {
  kind: "branch" | "tag" | string;
  shortName: string;
  qualifiedName: string;
  targetOid: string | null;
  recoveryHref: string;
};

export type RepositoryPathOverview = RepositorySummary & {
  viewerPermission: string | null;
  refName: string;
  resolvedRef: RepositoryResolvedRef;
  defaultBranchHref: string;
  recoveryHref: string;
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
  path: string;
  pathName: string;
  breadcrumbs: RepositoryPathBreadcrumb[];
  parentHref: string | null;
  entries: RepositoryTreeEntry[];
  readme: RepositoryFile | null;
  latestCommit: RepositoryLatestCommit | null;
  historyHref: string;
};

export type RepositoryBlobView = RepositorySummary & {
  viewerPermission: string | null;
  refName: string;
  resolvedRef: RepositoryResolvedRef;
  defaultBranchHref: string;
  recoveryHref: string;
  path: string;
  pathName: string;
  breadcrumbs: RepositoryPathBreadcrumb[];
  parentHref: string | null;
  file: RepositoryFile;
  language: string | null;
  isBinary: boolean;
  isLarge: boolean;
  lineCount: number;
  locCount: number;
  sizeLabel: string;
  mimeType: string;
  renderMode: "text" | "binary" | "large" | string;
  displayContent: string | null;
  latestCommit: RepositoryLatestCommit | null;
  latestPathCommit: RepositoryLatestCommit | null;
  historyHref: string;
  rawHref: string;
  downloadHref: string;
  rawApiHref: string;
  downloadApiHref: string;
  permalinkHref: string;
  symbols: RepositoryCodeSymbol[];
};

export type RepositoryCodeSymbol = {
  kind: string;
  name: string;
  lineNumber: number;
  preview: string;
};

export type RepositoryBlameCommit = {
  oid: string;
  shortOid: string;
  message: string;
  href: string;
  committedAt: string;
  authorLogin: string | null;
};

export type RepositoryBlameLine = {
  lineNumber: number;
  content: string;
  commit: RepositoryBlameCommit;
};

export type RepositoryBlameView = RepositoryBlobView & {
  lines: RepositoryBlameLine[];
};

export type RepositoryCommitHistoryItem = {
  oid: string;
  shortOid: string;
  message: string;
  href: string;
  committedAt: string;
  authorLogin: string | null;
};

export type RepositoryLanguageSummary = {
  language: string;
  color: string;
  byteCount: number;
  percentage: number;
};

export type RepositorySidebarMetadata = {
  about: string | null;
  websiteUrl: string | null;
  topics: string[];
  starsCount: number;
  watchersCount: number;
  forksCount: number;
  releasesCount: number;
  deploymentsCount: number;
  contributorsCount: number;
  languages: RepositoryLanguageSummary[];
};

export type RepositoryViewerState = {
  starred: boolean;
  watching: boolean;
  forkedRepositoryHref: string | null;
};

export type RepositorySocialState = RepositoryViewerState & {
  starsCount: number;
  watchersCount: number;
  forksCount: number;
};

export type RepositoryForkResult = {
  sourceRepositoryId: string;
  forkRepository: RepositorySummary;
  forkHref: string;
  social: RepositorySocialState;
};

export type RepositoryCloneUrls = {
  https: string;
  git: string;
  zip: string;
};

export type RepositoryRefSummary = {
  name: string;
  shortName: string;
  kind: "branch" | "tag" | string;
  href: string;
  samePathHref: string;
  active: boolean;
  targetShortOid: string | null;
  updatedAt: string;
};

export type RepositoryFileFinderItem = {
  path: string;
  name: string;
  kind: "file" | string;
  href: string;
  byteSize: number;
  language: string | null;
};

export type RepositoryFileFinderResult =
  ListEnvelope<RepositoryFileFinderItem> & {
    resolvedRef: RepositoryResolvedRef;
    defaultBranchHref: string;
    recoveryHref: string;
  };

export type RepositoryOverview = RepositorySummary & {
  viewerPermission: string | null;
  branchCount: number;
  tagCount: number;
  defaultBranchRef: {
    id: string;
    repository_id: string;
    name: string;
    kind: string;
    target_commit_id: string | null;
    created_at: string;
    updated_at: string;
  } | null;
  latestCommit: RepositoryLatestCommit | null;
  rootEntries: RepositoryTreeEntry[];
  files: RepositoryFile[];
  readme: RepositoryFile | null;
  sidebar: RepositorySidebarMetadata;
  viewerState: RepositoryViewerState;
  cloneUrls: RepositoryCloneUrls;
};

export type RepositoryMergeMethod = "squash" | "merge_commit" | "rebase";

export type RepositoryFeatureSettings = {
  issuesEnabled: boolean;
  projectsEnabled: boolean;
  wikiEnabled: boolean;
};

export type RepositoryMergeSettings = {
  allowSquash: boolean;
  allowMergeCommit: boolean;
  allowRebase: boolean;
  defaultMethod: RepositoryMergeMethod;
};

export type RepositoryDangerState = {
  isArchived: boolean;
  canArchive: boolean;
  canUnarchive: boolean;
  deleteSupported: boolean;
  transferSupported: boolean;
};

export type RepositorySettingsAuditEvent = {
  id: string;
  eventType: string;
  changedFields: string[];
  actorUserId: string;
  createdAt: string;
};

export type RepositorySettings = {
  id: string;
  ownerLogin: string;
  name: string;
  description: string | null;
  visibility: RepositoryVisibility;
  defaultBranch: string;
  isTemplate: boolean;
  allowForking: boolean;
  webCommitSignoffRequired: boolean;
  features: RepositoryFeatureSettings;
  merge: RepositoryMergeSettings;
  danger: RepositoryDangerState;
  branches: string[];
  viewerPermission: string;
  updatedAt: string;
  auditEvents: RepositorySettingsAuditEvent[];
};

export type RepositorySettingsPatch = {
  name?: string;
  description?: string | null;
  visibility?: RepositoryVisibility;
  defaultBranch?: string;
  isTemplate?: boolean;
  allowForking?: boolean;
  webCommitSignoffRequired?: boolean;
  isArchived?: boolean;
  features?: Partial<RepositoryFeatureSettings>;
  merge?: Partial<RepositoryMergeSettings>;
};

export type RepositorySettingsFetchResult =
  | { ok: true; settings: RepositorySettings }
  | { ok: false; status: number; code: string | null; message: string };

export type WritableRepositoryOwner = {
  ownerType: RepositoryOwnerType;
  id: string;
  login: string;
  displayName: string;
  avatarUrl: string | null;
};

export type RepositoryTemplateOption = {
  slug: string;
  displayName: string;
  description: string;
};

export type GitignoreTemplateOption = {
  slug: string;
  displayName: string;
  description: string;
};

export type LicenseTemplateOption = {
  slug: string;
  displayName: string;
  description: string;
};

export type RepositoryCreationOptions = {
  owners: WritableRepositoryOwner[];
  templates: RepositoryTemplateOption[];
  gitignoreTemplates: GitignoreTemplateOption[];
  licenseTemplates: LicenseTemplateOption[];
  suggestedName: string;
};

export type RepositoryNameAvailability = {
  ownerType: RepositoryOwnerType;
  ownerId: string;
  ownerLogin: string;
  requestedName: string;
  normalizedName: string;
  available: boolean;
  reason: string | null;
};

export type CreateRepositoryRequest = {
  ownerType: RepositoryOwnerType;
  ownerId: string;
  name: string;
  description?: string | null;
  visibility: Exclude<RepositoryVisibility, "internal">;
  defaultBranch?: string | null;
  initializeReadme?: boolean;
  templateSlug?: string | null;
  gitignoreTemplateSlug?: string | null;
  licenseTemplateSlug?: string | null;
};

export type CreatedRepository = RepositorySummary & {
  href: string;
  files?: RepositoryFile[];
  readme?: RepositoryFile | null;
};

export type RepositoryImportRequest = {
  sourceUrl: string;
  sourceUsername?: string | null;
  sourceToken?: string | null;
  sourcePassword?: string | null;
  ownerType: RepositoryOwnerType;
  ownerId: string;
  name: string;
  description?: string | null;
  visibility: Exclude<RepositoryVisibility, "internal">;
};

export type RepositoryImportStatusName =
  | "queued"
  | "importing"
  | "imported"
  | "failed";

export type RepositoryImportStatus = {
  id: string;
  repositoryId: string;
  requestedByUserId: string;
  source: {
    url: string;
    host: string;
    path: string;
  };
  status: RepositoryImportStatusName;
  progressMessage: string;
  errorCode: string | null;
  errorMessage: string | null;
  jobLeaseId: string | null;
  repositoryHref: string;
  statusHref: string;
  createdAt: string;
  updatedAt: string;
};

export type ApiErrorEnvelope = {
  error: {
    code: string;
    message: string;
  };
  status: number;
  details?: {
    field?: string;
    reason?: string;
    [key: string]: unknown;
  } | null;
};

export type ActionsWorkflowLatestRun = {
  id: string;
  runNumber: number;
  status: string;
  conclusion: string | null;
  createdAt: string;
};

export type ActionsWorkflowRailItem = {
  id: string;
  name: string;
  path: string;
  state: string;
  triggerEvents: string[];
  pinned: boolean;
  runCount: number;
  latestRun: ActionsWorkflowLatestRun | null;
};

export type ActionsActor = {
  id: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type ActionsRunPullRequest = {
  id: string;
  number: number;
  title: string;
};

export type ActionsJobSummary = {
  total: number;
  queued: number;
  inProgress: number;
  completed: number;
  cancelled: number;
  success: number;
  failure: number;
  skipped: number;
  timedOut: number;
};

export type ActionsRunListItem = {
  id: string;
  workflowId: string;
  workflowName: string;
  workflowPath: string;
  runNumber: number;
  displayTitle: string;
  status: string;
  conclusion: string | null;
  statusCategory: string;
  event: string;
  actor: ActionsActor | null;
  headBranch: string;
  headSha: string | null;
  shortSha: string | null;
  pullRequest: ActionsRunPullRequest | null;
  commitMessage: string | null;
  jobSummary: ActionsJobSummary;
  durationSeconds: number | null;
  isLive: boolean;
  startedAt: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ActionsRunFilters = {
  q: string | null;
  workflow: string | null;
  event: string | null;
  status: string | null;
  branch: string | null;
  actor: string | null;
  page: number;
  pageSize: number;
};

export type ActionsFilterOption = {
  value: string;
  label: string;
  count: number;
};

export type ActionsRunFilterOptions = {
  workflows: ActionsFilterOption[];
  events: ActionsFilterOption[];
  statuses: ActionsFilterOption[];
  branches: ActionsFilterOption[];
  actors: ActionsFilterOption[];
};

export type ActionsEmptyState = {
  hasWorkflows: boolean;
  hasRuns: boolean;
  message: string;
  newWorkflowHref: string;
};

export type RepositoryActionsDashboard = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  viewerPermission: string | null;
  workflows: ActionsWorkflowRailItem[];
  runs: ListEnvelope<ActionsRunListItem>;
  filters: ActionsRunFilters;
  filterOptions: ActionsRunFilterOptions;
  emptyState: ActionsEmptyState;
};

export type RepositoryActionsDashboardQuery = {
  q?: string | null;
  workflow?: string | null;
  event?: string | null;
  status?: string | null;
  branch?: string | null;
  actor?: string | null;
  page?: number | string | null;
  pageSize?: number | string | null;
};

export type WorkflowDispatchInput = {
  name: string;
  type: string;
  label: string;
  description: string | null;
  required: boolean;
  default: string | null;
  options: string[];
};

export type WorkflowDispatchSpec = {
  enabled: boolean;
  inputs: WorkflowDispatchInput[];
};

export type ActionsWorkflowDetailWorkflow = {
  id: string;
  name: string;
  path: string;
  state: string;
  triggerEvents: string[];
  sourceBranch: string;
  sourceSha: string | null;
  sourceBlobId: string | null;
  sourceHref: string;
  dispatch: WorkflowDispatchSpec;
  yamlParseError: string | null;
  yamlParsedAt: string;
  valid: boolean;
};

export type ActionsWorkflowRef = {
  name: string;
  shortName: string;
  kind: string;
  sha: string | null;
};

export type RepositoryActionsWorkflowDetail = Omit<
  RepositoryActionsDashboard,
  "filterOptions"
> & {
  workflow: ActionsWorkflowDetailWorkflow;
  filterOptions: Omit<ActionsRunFilterOptions, "workflows"> & {
    workflows: [];
  };
  refs: ActionsWorkflowRef[];
};

export type ActionsRunDetailWorkflow = {
  id: string;
  name: string;
  path: string;
  state: string;
  sourceBranch: string;
  sourceSha: string | null;
  sourceHref: string;
};

export type ActionsRunAttempt = {
  id: string | null;
  attemptNumber: number;
  status: string;
  conclusion: string | null;
  triggerKind: string;
  actor: ActionsActor | null;
  startedAt: string | null;
  completedAt: string | null;
  createdAt: string;
};

export type ActionsRunStepDetail = {
  id: string;
  number: number;
  name: string;
  status: string;
  conclusion: string | null;
  durationSeconds: number | null;
  startedAt: string | null;
  completedAt: string | null;
};

export type ActionsRunJobDetail = {
  id: string;
  name: string;
  groupName: string | null;
  attemptNumber: number;
  status: string;
  conclusion: string | null;
  runnerLabel: string | null;
  durationSeconds: number | null;
  logAvailable: boolean;
  logDeletedAt: string | null;
  steps: ActionsRunStepDetail[];
  startedAt: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ActionsRunAnnotation = {
  id: string;
  jobId: string | null;
  stepId: string | null;
  level: string;
  path: string | null;
  startLine: number | null;
  endLine: number | null;
  title: string | null;
  message: string;
  rawDetails: string | null;
  createdAt: string;
};

export type ActionsRunArtifact = {
  id: string;
  name: string;
  digest: string | null;
  sizeBytes: number;
  expiredAt: string | null;
  downloadAvailable: boolean;
  createdAt: string;
  updatedAt: string;
};

export type ActionsJobLogLine = {
  lineNumber: number;
  timestamp: string | null;
  content: string;
  anchor: string;
};

export type ActionsJobLog = {
  job: {
    id: string;
    runId: string;
    name: string;
    status: string;
    conclusion: string | null;
    logDeletedAt: string | null;
  };
  lines: ActionsJobLogLine[];
  total: number;
  page: number;
  pageSize: number;
  query: string | null;
  downloadHref: string;
};

export type ActionsJobLogStep = {
  id: string | null;
  number: number;
  name: string;
  status: string;
  conclusion: string | null;
  durationSeconds: number | null;
  startedAt: string | null;
  completedAt: string | null;
  lines: ListEnvelope<ActionsJobLogLine>;
  matchCount: number;
};

export type ActionsJobLogState = {
  available: boolean;
  status: number;
  reason: string | null;
  deletedAt: string | null;
  isLive: boolean;
  nextCursor: number | null;
};

export type ActionsJobLogSearchMatch = {
  lineNumber: number;
  stepId: string | null;
  stepNumber: number;
  anchor: string;
  preview: string;
};

export type ActionsJobLogSearch = {
  query: string | null;
  totalMatches: number;
  selectedMatch: number | null;
  matches: ActionsJobLogSearchMatch[];
};

export type ActionsJobLogOptions = {
  showTimestamps: boolean;
  rawLogs: boolean;
  wrapLines: boolean;
};

export type RepositoryActionsJobLogDetail = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  viewerPermission: string | null;
  workflow: ActionsRunDetailWorkflow;
  run: ActionsRunListItem;
  jobs: ActionsRunJobDetail[];
  job: ActionsRunJobDetail;
  steps: ActionsJobLogStep[];
  annotations: ActionsRunAnnotation[];
  logState: ActionsJobLogState;
  search: ActionsJobLogSearch;
  options: ActionsJobLogOptions;
  downloadHref: string;
  runArchiveHref: string;
};

export type RepositoryActionsJobLogDetailQuery = {
  q?: string | null;
  selectedMatch?: number | null;
  timestamps?: boolean | null;
  raw?: boolean | null;
  page?: number | null;
  pageSize?: number | null;
};

export type ActionsArtifactDownload = {
  artifactId: string;
  name: string;
  filename: string;
  downloadUrl: string;
  storageKey: string;
  expiresAt: string;
};

export type ActionsRunActionState = {
  canRerun: boolean;
  canRerunFailed: boolean;
  canCancel: boolean;
  canDeleteLogs: boolean;
  disabledReason: string | null;
};

export type RepositoryActionsRunDetail = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  viewerPermission: string | null;
  workflow: ActionsRunDetailWorkflow;
  run: ActionsRunListItem;
  attempts: ActionsRunAttempt[];
  jobs: ActionsRunJobDetail[];
  annotations: ActionsRunAnnotation[];
  artifacts: ActionsRunArtifact[];
  actionState: ActionsRunActionState;
};

export type DashboardTopRepository = {
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  primaryLanguage: string | null;
  primaryLanguageColor: string | null;
  updatedAt: string;
  lastVisitedAt: string | null;
  href: string;
};

export type AppShellRepository = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  href: string;
  updatedAt: string;
  lastVisitedAt: string | null;
};

export type AppShellOrganization = {
  id: string;
  slug: string;
  displayName: string;
  role: string;
  href: string;
};

export type AppShellTeam = {
  id: string;
  organizationId: string;
  organizationSlug: string;
  slug: string;
  name: string;
  role: string;
  href: string;
};

export type AppShellQuickLink = {
  label: string;
  href: string;
  kind: string;
};

export type AppShellContext = {
  user: AuthUser;
  unreadNotificationCount: number;
  recentRepositories: AppShellRepository[];
  organizations: AppShellOrganization[];
  teams: AppShellTeam[];
  quickLinks: AppShellQuickLink[];
};

export type ListEnvelope<T> = {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
};

export type DashboardHintDismissal = {
  id: string;
  userId: string;
  hintKey: string;
  dismissedAt: string;
};

export type DashboardSummary = {
  user: AuthUser;
  repositories: ListEnvelope<RepositorySummary>;
  topRepositories: ListEnvelope<DashboardTopRepository>;
  hasRepositories: boolean;
  recentActivity: DashboardActivityItem[];
  feedEvents: DashboardFeedEvent[];
  feedPreferences: DashboardFeedPreferences;
  supportedFeedEventTypes: DashboardFeedEventType[];
  assignedIssues: DashboardIssueSummary[];
  reviewRequests: DashboardReviewRequest[];
  dismissedHints: DashboardHintDismissal[];
};

export type DashboardFeedTab = "following" | "for_you";

export type DashboardFeedEventType =
  | "star"
  | "follow"
  | "repository_create"
  | "help_wanted_issue"
  | "help_wanted_pull_request"
  | "push"
  | "fork"
  | "release";

export type DashboardFeedEvent = {
  id: string;
  eventType: DashboardFeedEventType;
  title: string;
  excerpt: string | null;
  occurredAt: string;
  actorLogin: string;
  actorAvatarUrl: string | null;
  repositoryName: string;
  repositoryHref: string;
  targetHref: string;
  actionSummary: string;
};

export type DashboardFeedPreferences = {
  feedTab: DashboardFeedTab;
  eventTypes: DashboardFeedEventType[];
};

export type DashboardActivityItem = {
  id: string;
  kind: "repository" | "commit" | "issue" | "pull_request" | string;
  title: string;
  number: number;
  state: "open" | "closed" | "merged" | string;
  repositoryName: string;
  repositoryHref: string;
  href: string;
  occurredAt: string;
  description: string | null;
  actorLogin: string;
  actorAvatarUrl: string | null;
};

export type DashboardIssueSummary = {
  id: string;
  title: string;
  repositoryName: string;
  number: number;
  href: string;
  updatedAt: string;
};

export type DashboardReviewRequest = {
  id: string;
  title: string;
  repositoryName: string;
  number: number;
  href: string;
  updatedAt: string;
};

export type IssueState = "open" | "closed";

export type IssueListLabel = {
  id: string;
  name: string;
  color: string;
  description: string | null;
};

export type IssueListUser = {
  id: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type IssueListMilestone = {
  id: string;
  title: string;
  state: IssueState;
};

export type IssueSort =
  | "updated-desc"
  | "updated-asc"
  | "created-desc"
  | "created-asc"
  | "comments-desc"
  | "comments-asc"
  | "best-match";

export type IssueListMetadataOption = {
  id: string;
  name: string;
  description: string | null;
  count: number;
  disabledReason: string | null;
};

export type LinkedPullRequestHint = {
  number: number;
  state: string;
  href: string;
};

export type IssueListItem = {
  id: string;
  repositoryId: string;
  repositoryOwner: string;
  repositoryName: string;
  number: number;
  title: string;
  body: string | null;
  state: IssueState;
  author: IssueListUser;
  labels: IssueListLabel[];
  milestone: IssueListMilestone | null;
  assignees: IssueListUser[];
  commentCount: number;
  linkedPullRequest: LinkedPullRequestHint | null;
  href: string;
  locked: boolean;
  createdAt: string;
  updatedAt: string;
  closedAt: string | null;
};

export type IssueAttachmentMetadata = {
  id: string;
  fileName: string;
  byteSize: number;
  contentType: string | null;
  storageStatus: string;
  createdAt: string;
};

export type IssueSubscriptionState = {
  subscribed: boolean;
  reason: string;
};

export type ReactionContent =
  | "thumbs_up"
  | "thumbs_down"
  | "laugh"
  | "hooray"
  | "confused"
  | "heart"
  | "rocket"
  | "eyes";

export type ReactionSummary = {
  content: ReactionContent;
  count: number;
  viewerReacted: boolean;
};

export type IssueDetailView = {
  id: string;
  repositoryId: string;
  repositoryOwner: string;
  repositoryName: string;
  number: number;
  title: string;
  body: string | null;
  bodyHtml: string;
  state: IssueState;
  author: IssueListUser;
  labels: IssueListLabel[];
  milestone: IssueListMilestone | null;
  assignees: IssueListUser[];
  participants: IssueListUser[];
  attachments: IssueAttachmentMetadata[];
  commentCount: number;
  linkedPullRequest: LinkedPullRequestHint | null;
  href: string;
  locked: boolean;
  createdAt: string;
  updatedAt: string;
  closedAt: string | null;
  viewerPermission: string | null;
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
  };
  subscription: IssueSubscriptionState;
  reactions: ReactionSummary[];
  metadataOptions: {
    labels: IssueListLabel[];
    assignees: IssueListUser[];
    milestones: IssueListMilestone[];
  };
};

export type UpdateIssueMetadataRequest = {
  labelIds: string[];
  assigneeUserIds: string[];
  milestoneId: string | null;
};

export type IssueTimelineComment = {
  id: string;
  body: string;
  bodyHtml: string;
  isMinimized: boolean;
  reactions: ReactionSummary[];
  createdAt: string;
  updatedAt: string;
};

export type IssueTimelineItem = {
  id: string;
  eventType: string;
  actor: IssueListUser | null;
  comment: IssueTimelineComment | null;
  metadata: Record<string, unknown>;
  createdAt: string;
};

export type PullRequestTimelineComment = IssueTimelineComment;

export type PullRequestTimelineItem = {
  id: string;
  eventType: string;
  actor: IssueListUser | null;
  comment: PullRequestTimelineComment | null;
  metadata: Record<string, unknown>;
  createdAt: string;
};

export type IssueListFilters = {
  query: string;
  state: IssueState;
  author: string | null;
  excludedAuthor: string | null;
  labels: string[];
  excludedLabels: string[];
  noLabels: boolean;
  milestone: string | null;
  noMilestone: boolean;
  assignee: string | null;
  noAssignee: boolean;
  project: string | null;
  issueType: string | null;
  sort: IssueSort;
};

export type IssueListPreferences = {
  dismissedContributorBanner: boolean;
  dismissedContributorBannerAt: string | null;
};

export type IssueListView = ListEnvelope<IssueListItem> & {
  openCount: number;
  closedCount: number;
  counts: {
    open: number;
    closed: number;
  };
  filters: IssueListFilters;
  filterOptions: {
    labels: IssueListLabel[];
    users: IssueListUser[];
    milestones: IssueListMilestone[];
    projects: IssueListMetadataOption[];
    issueTypes: IssueListMetadataOption[];
  };
  viewerPermission: string | null;
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
  };
  preferences: IssueListPreferences;
};

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

function isIssueListView(value: unknown): value is IssueListView {
  if (!isObjectRecord(value)) {
    return false;
  }

  const filters = value.filters;
  const filterOptions = value.filterOptions;
  const preferences = value.preferences;
  return (
    Array.isArray(value.items) &&
    typeof value.total === "number" &&
    typeof value.page === "number" &&
    typeof value.pageSize === "number" &&
    typeof value.openCount === "number" &&
    typeof value.closedCount === "number" &&
    isObjectRecord(value.counts) &&
    isObjectRecord(filters) &&
    typeof filters.query === "string" &&
    typeof filters.state === "string" &&
    Array.isArray(filters.labels) &&
    Array.isArray(filters.excludedLabels) &&
    typeof filters.sort === "string" &&
    isObjectRecord(filterOptions) &&
    Array.isArray(filterOptions.labels) &&
    Array.isArray(filterOptions.users) &&
    Array.isArray(filterOptions.milestones) &&
    Array.isArray(filterOptions.projects) &&
    Array.isArray(filterOptions.issueTypes) &&
    isObjectRecord(value.repository) &&
    isObjectRecord(preferences) &&
    typeof preferences.dismissedContributorBanner === "boolean"
  );
}

export type PullRequestState = "open" | "closed" | "merged";

export type PullRequestSort =
  | "best-match"
  | "updated-desc"
  | "updated-asc"
  | "created-desc"
  | "created-asc"
  | "comments-desc"
  | "comments-asc"
  | "reactions-desc"
  | "reactions-thumbs_up-desc"
  | "reactions-thumbs_down-desc"
  | "reactions-laugh-desc"
  | "reactions-hooray-desc"
  | "reactions-confused-desc"
  | "reactions-heart-desc"
  | "reactions-rocket-desc"
  | "reactions-eyes-desc";

export type LinkedIssueHint = {
  number: number;
  state: string;
  title: string;
  href: string;
};

export type PullRequestReviewSummary = {
  state: string;
  required: boolean;
  requestedReviewers: IssueListUser[];
  reviewerCount: number;
};

export type PullRequestChecksSummary = {
  status: string;
  conclusion: string | null;
  totalCount: number;
  completedCount: number;
  failedCount: number;
};

export type PullRequestTaskProgress = {
  completed: number;
  total: number;
};

export type PullRequestListItem = {
  id: string;
  repositoryId: string;
  repositoryOwner: string;
  repositoryName: string;
  number: number;
  title: string;
  body: string | null;
  state: PullRequestState;
  isDraft: boolean;
  author: IssueListUser;
  authorRole: string;
  labels: IssueListLabel[];
  milestone: IssueListMilestone | null;
  commentCount: number;
  linkedIssues: LinkedIssueHint[];
  review: PullRequestReviewSummary;
  checks: PullRequestChecksSummary;
  taskProgress: PullRequestTaskProgress;
  headRef: string;
  baseRef: string;
  href: string;
  checksHref: string;
  reviewsHref: string;
  commentsHref: string;
  linkedIssuesHref: string;
  createdAt: string;
  updatedAt: string;
  closedAt: string | null;
  mergedAt: string | null;
};

export type PullRequestListFilters = {
  query: string;
  state: PullRequestState;
  author: string | null;
  labels: string[];
  milestone: string | null;
  noMilestone: boolean;
  assignee: string | null;
  noAssignee: boolean;
  project: string | null;
  review: string | null;
  checks: string | null;
  sort: PullRequestSort;
};

export type PullRequestListPreferences = {
  dismissedContributorBanner: boolean;
  dismissedContributorBannerAt: string | null;
};

export type PullRequestListView = ListEnvelope<PullRequestListItem> & {
  openCount: number;
  closedCount: number;
  mergedCount: number;
  counts: {
    open: number;
    closed: number;
    merged: number;
  };
  filters: PullRequestListFilters;
  filterOptions: {
    labels: IssueListLabel[];
    users: IssueListUser[];
    milestones: IssueListMilestone[];
    projects: IssueListMetadataOption[];
    reviewStates: string[];
    checkStates: string[];
    sortOptions: PullRequestSort[];
  };
  viewerPermission: string | null;
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  preferences: PullRequestListPreferences;
};

export type PullRequestCompareStatus =
  | "same_ref"
  | "no_diff"
  | "ahead"
  | "diverged";

export type PullRequestCompareRef = {
  repository: PullRequestListView["repository"];
  name: string;
  shortName: string;
  kind: string;
  oid: string;
  commitId: string;
  href: string;
};

export type PullRequestCompareCommit = {
  id: string;
  oid: string;
  shortOid: string;
  message: string;
  authorLogin: string | null;
  committedAt: string;
  href: string;
};

export type PullRequestCompareFile = {
  path: string;
  status: "added" | "modified" | "removed";
  additions: number;
  deletions: number;
  byteSize: number;
  blobOid: string | null;
  href: string;
};

export type PullRequestTemplateOption = {
  slug: string;
  name: string;
  body: string;
};

export type PullRequestCreateOptions = {
  canCreate: boolean;
  templates: PullRequestTemplateOption[];
  labels: IssueListLabel[];
  users: IssueListUser[];
  milestones: IssueListMilestone[];
  forkRepositories: PullRequestCompareRepositoryOption[];
};

export type PullRequestCompareRepositoryOption = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  defaultBranch: string;
  href: string;
  compareHref: string;
  isBase: boolean;
  isSelectedHead: boolean;
};

export type PullRequestCompareView = {
  repository: PullRequestListView["repository"];
  viewerPermission: string | null;
  base: PullRequestCompareRef;
  head: PullRequestCompareRef;
  status: PullRequestCompareStatus;
  aheadBy: number;
  behindBy: number;
  totalCommits: number;
  totalFiles: number;
  commits: PullRequestCompareCommit[];
  files: PullRequestCompareFile[];
  additions: number;
  deletions: number;
  defaultBranchHref: string;
  pullListHref: string;
  compareHref: string;
  swapHref: string;
  createOptions: PullRequestCreateOptions;
};

export type PullRequestDiffReviewSettings = {
  view: "unified" | "split" | string;
  whitespace: "show" | "hide" | string;
  commit: string | null;
  filter: string | null;
  page: number;
  pageSize: number;
};

export type PullRequestDiffFileTreeItem = {
  id: string;
  path: string;
  status: "added" | "modified" | "removed" | "renamed" | string;
  additions: number;
  deletions: number;
  viewed: boolean;
  versionKey: string;
  href: string;
};

export type PullRequestDiffLine = {
  kind: "context" | "added" | "removed";
  oldLine: number | null;
  newLine: number | null;
  content: string;
  position: number;
  commentCount: number;
};

export type PullRequestDiffHunk = {
  id: string;
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: PullRequestDiffLine[];
};

export type PullRequestDiffReviewComment = {
  id: string;
  author: IssueListUser;
  body: string;
  bodyHtml: string;
  path: string;
  side: "left" | "right" | string;
  oldLine: number | null;
  newLine: number | null;
  position: number | null;
  state: string;
  createdAt: string;
  updatedAt: string;
};

export type PullRequestDiffFile = {
  id: string;
  path: string;
  status: "added" | "modified" | "removed" | "renamed" | string;
  additions: number;
  deletions: number;
  byteSize: number;
  blobOid: string | null;
  language: string | null;
  viewed: boolean;
  viewedAt: string | null;
  versionKey: string;
  href: string;
  hunks: PullRequestDiffHunk[];
  comments: PullRequestDiffReviewComment[];
};

export type PullRequestDiffPendingReview = {
  draftId: string | null;
  commentCount: number;
  summaryBody: string | null;
  reviewState: "commented" | "approved" | "changes_requested" | string;
};

export type PullRequestViewedFileState = {
  fileId: string;
  path: string;
  viewed: boolean;
  viewedAt: string | null;
  versionKey: string;
};

export type SubmitPullRequestReviewRequest = {
  body: string | null;
  state: "commented" | "approved" | "changes_requested";
};

export type PullRequestSubmittedReview = {
  id: string;
  reviewer: IssueListUser;
  state: string;
  body: string | null;
  submittedAt: string;
  publishedCommentCount: number;
  pendingReview: PullRequestDiffPendingReview;
};

export type CreatePullRequestReviewDraftCommentRequest = {
  fileId: string;
  body: string;
  side: "left" | "right" | string;
  oldLine: number | null;
  newLine: number | null;
  position: number;
};

export type UpdatePullRequestReviewDraftCommentRequest = {
  body: string;
};

export type PullRequestDiffReviewView = {
  pullRequest: PullRequestDetailView;
  settings: PullRequestDiffReviewSettings;
  totalFiles: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
  fileTree: PullRequestDiffFileTreeItem[];
  files: PullRequestDiffFile[];
  commits: PullRequestCompareCommit[];
  pendingReview: PullRequestDiffPendingReview;
};

export type PullRequestDetailView = {
  id: string;
  issueId: string;
  repository: PullRequestListView["repository"];
  number: number;
  title: string;
  body: string | null;
  bodyHtml: string;
  state: PullRequestState;
  isDraft: boolean;
  author: IssueListUser;
  authorRole: string;
  headRef: string;
  baseRef: string;
  labels: IssueListLabel[];
  milestone: IssueListMilestone | null;
  assignees: IssueListUser[];
  requestedReviewers: IssueListUser[];
  latestReviews: Array<{
    reviewer: IssueListUser;
    state: string;
    submittedAt: string;
  }>;
  linkedIssues: LinkedIssueHint[];
  participants: IssueListUser[];
  review: PullRequestReviewSummary;
  checks: PullRequestChecksSummary;
  taskProgress: PullRequestTaskProgress;
  stats: {
    commits: number;
    files: number;
    additions: number;
    deletions: number;
    comments: number;
  };
  subscription: {
    subscribed: boolean;
    reason: string;
  };
  mergeability: {
    state: "ready" | "blocked" | "closed" | "merged" | string;
    canMerge: boolean;
    canClose: boolean;
    canReopen: boolean;
    canMarkReady: boolean;
    defaultMethod: MergeMethod;
    methods: MergeMethod[];
    canDeleteHeadBranch?: boolean;
    defaultCommitTitle?: string | null;
    defaultCommitBody?: string | null;
    branchProtection: {
      protected: boolean;
      pattern: string | null;
      requiredApprovingReviewCount: number;
      requiresUpToDateBranch: boolean;
      requiredStatusChecks: string[];
    };
    blockers: Array<{
      code: string;
      message: string;
      severity: string;
    }>;
    summary: string;
  };
  metadataOptions: {
    labels: IssueListLabel[];
    assignees: IssueListUser[];
    milestones: IssueListMilestone[];
  };
  href: string;
  commitsHref: string;
  checksHref: string;
  filesHref: string;
  createdAt: string;
  updatedAt: string;
  closedAt: string | null;
  mergedAt: string | null;
  viewerPermission: string | null;
};

export type MergeMethod = "squash" | "merge_commit" | "rebase";

export type UpdatePullRequestMetadataRequest = {
  labelIds: string[];
  assigneeUserIds: string[];
  milestoneId: string | null;
};

export type PullRequestSubscriptionState =
  PullRequestDetailView["subscription"];

export type CreatePullRequestRequest = {
  title: string;
  body?: string | null;
  headRef: string;
  baseRef: string;
  headRepositoryId?: string | null;
  headOwner?: string | null;
  headRepo?: string | null;
  isDraft?: boolean;
  labelIds?: string[];
  milestoneId?: string | null;
  assigneeUserIds?: string[];
  reviewerUserIds?: string[];
  templateSlug?: string | null;
};

export type CreatedPullRequest = {
  pull_request: {
    id: string;
    number: number;
    title: string;
    body: string | null;
    state: "open" | "closed" | "merged";
    is_draft: boolean;
    head_ref: string;
    base_ref: string;
  };
  issue: CreatedIssue;
  href: string;
};

export type CreatedIssue = {
  id: string;
  repository_id: string;
  number: number;
  title: string;
  body: string | null;
  state: IssueState;
  author_user_id: string;
  milestone_id: string | null;
  locked: boolean;
  closed_by_user_id: string | null;
  closed_at: string | null;
  created_at: string;
  updated_at: string;
  href?: string;
};

export type CreateIssueRequest = {
  title: string;
  body?: string | null;
  templateId?: string | null;
  templateSlug?: string | null;
  fieldValues?: Record<string, string>;
  milestoneId?: string | null;
  labelIds?: string[];
  assigneeUserIds?: string[];
  attachments?: IssueAttachmentInput[];
};

export type IssueAttachmentInput = {
  fileName: string;
  byteSize: number;
  contentType?: string | null;
};

export type IssueFormField = {
  id: string;
  templateId: string;
  fieldKey: string;
  label: string;
  fieldType: "markdown" | "textarea" | "input" | string;
  description: string | null;
  placeholder: string | null;
  value: string | null;
  required: boolean;
  displayOrder: number;
  createdAt: string;
  updatedAt: string;
};

export type IssueTemplate = {
  id: string;
  repositoryId: string;
  slug: string;
  name: string;
  description: string | null;
  titlePrefill: string | null;
  body: string;
  issueType: string | null;
  formFields: IssueFormField[];
  defaultLabelIds: string[];
  defaultAssigneeUserIds: string[];
  createdAt: string;
  updatedAt: string;
};

export type IssueTemplateList = {
  items: IssueTemplate[];
};

export type RepositoryIssueListQuery = {
  q?: string;
  state?: IssueState;
  author?: string;
  excludedAuthor?: string;
  labels?: string[];
  excludedLabels?: string[];
  noLabels?: boolean;
  milestone?: string;
  noMilestone?: boolean;
  assignee?: string;
  noAssignee?: boolean;
  project?: string;
  issueType?: string;
  sort?: string;
  page?: number;
  pageSize?: number;
};

export type RepositoryPullRequestListQuery = {
  q?: string;
  state?: PullRequestState;
  author?: string;
  labels?: string[];
  milestone?: string;
  noMilestone?: boolean;
  assignee?: string;
  noAssignee?: boolean;
  project?: string;
  review?: string;
  checks?: string;
  sort?: string;
  order?: "asc" | "desc";
  page?: number;
  pageSize?: number;
};

export type RepositoryPullRequestDiffQuery = {
  view?: "unified" | "split" | string;
  whitespace?: "show" | "hide" | string;
  commit?: string;
  filter?: string;
  page?: number;
  pageSize?: number;
};

export type RenderMarkdownRequest = {
  markdown: string;
  repositoryId?: string | null;
  owner?: string | null;
  repo?: string | null;
  ref?: string | null;
  enableTaskToggles?: boolean;
};

export type RenderedMarkdown = {
  contentSha: string;
  html: string;
  cached: boolean;
};

export type HighlightToken = {
  text: string;
  className: string;
};

export type HighlightedLine = {
  number: number;
  text: string;
  tokens: HighlightToken[];
};

export type CodeSymbol = {
  name: string;
  kind: string;
  line: number;
};

export type LanguageOption = {
  id: string;
  label: string;
};

export type HighlightCodeRequest = {
  source: string;
  path?: string | null;
  sha?: string | null;
  repositoryId?: string | null;
  language?: string | null;
};

export type HighlightedFile = {
  sha: string;
  path: string;
  language: string;
  cached: boolean;
  lines: HighlightedLine[];
  symbols: CodeSymbol[];
  supportedLanguages: LanguageOption[];
};

export type SearchResultType =
  | "repositories"
  | "code"
  | "issues"
  | "pull_requests"
  | "commits"
  | "users"
  | "organizations"
  | "discussions";

export type SearchDocumentKind =
  | "repository"
  | "code"
  | "commit"
  | "issue"
  | "pull_request"
  | "user"
  | "organization"
  | "package";

export type SearchDocument = {
  id: string;
  repository_id: string | null;
  owner_user_id: string | null;
  owner_organization_id: string | null;
  kind: SearchDocumentKind;
  resource_id: string;
  title: string;
  body: string;
  path: string | null;
  language: string | null;
  branch: string | null;
  visibility: RepositoryVisibility;
  metadata: Record<string, unknown>;
  indexed_at: string;
  created_at: string;
  updated_at: string;
};

export type GlobalSearchResult = {
  document: SearchDocument;
  rank: number;
  type: SearchResultType | string;
  href: string;
  title: string;
  summary: string | null;
  owner_login: string | null;
  repository_name: string | null;
  display_name: string | null;
  avatar_url: string | null;
  visibility: RepositoryVisibility;
  updated_at: string;
  snippet: {
    path: string;
    branch: string;
    line_number: number | null;
    fragment: string;
    language: string | null;
    match_ranges: { start: number; end: number }[];
  } | null;
  snippets: {
    path: string;
    branch: string;
    line_number: number | null;
    fragment: string;
    language: string | null;
    match_ranges: { start: number; end: number }[];
  }[];
  match_count: number;
  hidden_match_count: number;
  blob_href: string | null;
  commit: {
    oid: string;
    short_oid: string;
    message_title: string;
    message_body: string | null;
    author_login: string | null;
    committed_at: string | null;
  } | null;
};

export type GlobalSearchQuery = {
  query: string;
  type: SearchResultType | string;
  page?: number;
  pageSize?: number;
};

export type CodeSearchQuery = {
  query: string;
  page?: number;
  pageSize?: number;
};

export type CodeSearchTypeCount = {
  resultType: SearchResultType | string;
  label: string;
  count: number;
};

export type CodeSearchFacetValue = {
  value: string;
  label: string;
  count: number;
  selected: boolean;
};

export type CodeSearchChip = {
  qualifier: string;
  value: string;
  label: string;
  removeQuery: string;
};

export type CodeSearchDiagnostic = {
  code: string;
  message: string;
  qualifier: string | null;
};

export type CodeSearchResponse = ListEnvelope<GlobalSearchResult> & {
  typeCounts: CodeSearchTypeCount[];
  facets: {
    languages: CodeSearchFacetValue[];
    paths: CodeSearchFacetValue[];
  };
  activeChips: CodeSearchChip[];
  queryDurationMs: number;
  diagnostics: CodeSearchDiagnostic[];
};

export type CollaborationSearchQuery = {
  query: string;
  type: "issues" | "pull_requests" | "pullrequests" | "pulls" | string;
  page?: number;
  pageSize?: number;
  sort?: string;
};

export type CollaborationSearchSortOption = {
  value: string;
  label: string;
  selected: boolean;
};

export type CollaborationSearchFacetValue = CodeSearchFacetValue;

export type CollaborationSearchResponse = ListEnvelope<
  CollaborationSearchResult | GlobalSearchResult
> & {
  typeCounts: CodeSearchTypeCount[];
  facets: {
    states: CollaborationSearchFacetValue[];
    owners?: CollaborationSearchFacetValue[];
    labels: CollaborationSearchFacetValue[];
    milestones: CollaborationSearchFacetValue[];
    assignees: CollaborationSearchFacetValue[];
    reviewers?: CollaborationSearchFacetValue[];
  };
  activeChips: CodeSearchChip[];
  sort?: {
    selected: string;
    label: string;
    options: CollaborationSearchSortOption[];
  };
  sortOptions?: CollaborationSearchSortOption[];
  activeSort?: string;
  queryDurationMs: number;
  diagnostics?: CodeSearchDiagnostic[];
};

export type CollaborationSearchResult = {
  id: string;
  type: "issues" | "pull_requests" | string;
  href: string;
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    href: string;
  };
  number: number;
  title: string;
  state: string;
  closeReason: string | null;
  labels: { name: string; color: string; description: string | null }[];
  author: {
    id: string;
    login: string;
    displayName: string | null;
    avatarUrl: string | null;
  } | null;
  assignees: {
    id: string;
    login: string;
    displayName: string | null;
    avatarUrl: string | null;
  }[];
  milestone: { id: string; title: string; state: string } | null;
  linkedPullRequest: boolean;
  headRef: string | null;
  baseRef: string | null;
  commentCount: number;
  interactionCount: number;
  openedAt: string;
  updatedAt: string;
  closedAt: string | null;
  snippets: {
    field: string;
    fragment: string;
    matchRanges: { start: number; end: number }[];
  }[];
  rank: number;
};

const DEFAULT_API_URL = "http://localhost:3016";

export function apiBaseUrl(): string {
  return (
    process.env.API_URL ??
    process.env.NEXT_PUBLIC_API_URL ??
    DEFAULT_API_URL
  ).replace(/\/$/, "");
}

export function sanitizeNextPath(value: string | string[] | undefined): string {
  const candidate = Array.isArray(value) ? value[0] : value;
  if (
    !candidate?.startsWith("/") ||
    candidate.startsWith("//") ||
    candidate.includes("\\") ||
    candidate.includes("\n") ||
    candidate.includes("\r")
  ) {
    return "/dashboard";
  }
  return candidate;
}

export function googleStartUrl(nextPath: string): string {
  const url = new URL("/api/auth/google/start", apiBaseUrl());
  url.searchParams.set("next", sanitizeNextPath(nextPath));
  return url.toString();
}

export async function getSessionFromCookie(
  cookie: string | null | undefined,
): Promise<AuthSession> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/auth/me`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return { authenticated: false, user: null };
  }

  if (!response.ok) {
    return { authenticated: false, user: null };
  }

  return (await response.json()) as AuthSession;
}

export async function getSessionFromHeaders(
  requestHeaders: Headers,
): Promise<AuthSession> {
  return getSessionFromCookie(requestHeaders.get("cookie"));
}

export async function getAppShellContextFromCookie(
  cookie: string | null | undefined,
): Promise<AppShellContext | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/app-shell`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as AppShellContext;
}

export async function getPublicUserProfileFromCookie(
  cookie: string | null | undefined,
  username: string,
  options: { year?: number } = {},
): Promise<PublicUserProfile | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/profile`,
    );
    if (options.year) {
      url.searchParams.set("year", String(options.year));
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as PublicUserProfile;
}

export async function getPublicOrganizationProfileFromCookie(
  cookie: string | null | undefined,
  org: string,
): Promise<PublicOrganizationProfile | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/profile`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as PublicOrganizationProfile;
}

export async function getOrganizationRepositoriesFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: OrganizationRepositoryListQuery = {},
): Promise<OrganizationRepositoryList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/repositories`,
    );
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.type) {
      url.searchParams.set("type", query.type);
    }
    if (query.language) {
      url.searchParams.set("language", query.language);
    }
    if (query.sort) {
      url.searchParams.set("sort", query.sort);
    }
    if (query.density) {
      url.searchParams.set("density", query.density);
    }
    if (query.page) {
      url.searchParams.set("page", String(query.page));
    }
    if (query.pageSize) {
      url.searchParams.set("pageSize", String(query.pageSize));
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as OrganizationRepositoryList;
}

export async function getOrganizationPeopleFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: OrganizationPeopleListQuery = {},
): Promise<OrganizationPeopleList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people`,
    );
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.page) {
      url.searchParams.set("page", String(query.page));
    }
    if (query.pageSize) {
      url.searchParams.set("pageSize", String(query.pageSize));
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as OrganizationPeopleList;
}

export async function getProfileRepositoriesFromCookie(
  cookie: string | null | undefined,
  username: string,
  query: ProfileRepositoryListQuery = {},
): Promise<ProfileRepositoryList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/repositories`,
    );
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.type) {
      url.searchParams.set("type", query.type);
    }
    if (query.language) {
      url.searchParams.set("language", query.language);
    }
    if (query.sort) {
      url.searchParams.set("sort", query.sort);
    }
    if (query.page) {
      url.searchParams.set("page", String(query.page));
    }
    if (query.pageSize) {
      url.searchParams.set("pageSize", String(query.pageSize));
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as ProfileRepositoryList;
}

export async function getProfileStarsFromCookie(
  cookie: string | null | undefined,
  username: string,
  query: ProfileRepositoryListQuery = {},
): Promise<ProfileRepositoryList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/stars`,
    );
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.language) {
      url.searchParams.set("language", query.language);
    }
    if (query.sort) {
      url.searchParams.set("sort", query.sort);
    }
    if (query.page) {
      url.searchParams.set("page", String(query.page));
    }
    if (query.pageSize) {
      url.searchParams.set("pageSize", String(query.pageSize));
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as ProfileRepositoryList;
}

export async function setUserFollowFromCookie(
  cookie: string | null | undefined,
  username: string,
  following: boolean,
): Promise<ProfileActionState> {
  const response = await fetch(
    `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/follow`,
    {
      method: following ? "PUT" : "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Profile follow update failed", {
      cause: body,
    });
  }

  return (await response.json()) as ProfileActionState;
}

export async function blockUserFromCookie(
  cookie: string | null | undefined,
  username: string,
  reason?: string,
): Promise<ProfileActionState> {
  const response = await fetch(
    `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/block`,
    {
      method: "PUT",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ reason }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Profile block failed", {
      cause: body,
    });
  }

  return (await response.json()) as ProfileActionState;
}

export async function reportUserFromCookie(
  cookie: string | null | undefined,
  username: string,
  request: ReportUserRequest,
): Promise<ProfileReport> {
  const response = await fetch(
    `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/reports`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Profile report failed", {
      cause: body,
    });
  }

  return (await response.json()) as ProfileReport;
}

export function globalSearchPath(query: GlobalSearchQuery): string {
  const params = new URLSearchParams();
  params.set("q", query.query);
  params.set("type", query.type);
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  return `/api/search?${params.toString()}`;
}

export function codeSearchPath(query: CodeSearchQuery): string {
  return globalSearchPath({
    query: query.query,
    type: "code",
    page: query.page,
    pageSize: query.pageSize,
  });
}

export function collaborationSearchPath(
  query: CollaborationSearchQuery,
): string {
  const params = new URLSearchParams();
  params.set("q", query.query);
  params.set("type", query.type);
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  if (query.sort) {
    params.set("sort", query.sort);
  }
  return `/api/search?${params.toString()}`;
}

export function searchSuggestionsPath(query: SearchSuggestionsQuery = {}) {
  const params = new URLSearchParams();
  if (query.query?.trim()) {
    params.set("q", query.query.trim());
  }
  if (query.scope?.trim()) {
    params.set("scope", query.scope.trim());
  }
  if (query.limit) {
    params.set("limit", String(query.limit));
  }
  const paramString = params.toString();
  return paramString
    ? `/api/search/suggestions?${paramString}`
    : "/api/search/suggestions";
}

export function savedSearchesPath() {
  return "/api/search/saved-searches";
}

export function savedSearchPath(id: string) {
  return `/api/search/saved-searches/${encodeURIComponent(id)}`;
}

export async function getSearchSuggestionsFromCookie(
  cookie: string | null | undefined,
  query: SearchSuggestionsQuery = {},
): Promise<SearchSuggestionDashboard | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${searchSuggestionsPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Search suggestions are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "suggestions_failed",
          message: "Search suggestions failed.",
        },
        status: response.status,
      }
    );
  }

  return body as SearchSuggestionDashboard;
}

export async function createSavedSearchFromCookie(
  cookie: string | null | undefined,
  input: CreateSavedSearchRequest,
): Promise<SavedSearchSuggestion | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${savedSearchesPath()}`, {
      body: JSON.stringify(input),
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      method: "POST",
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Saved search could not be created.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "saved_search_failed",
          message: "Saved search could not be created.",
        },
        status: response.status,
      }
    );
  }

  return body as SavedSearchSuggestion;
}

export async function deleteSavedSearchFromCookie(
  cookie: string | null | undefined,
  id: string,
): Promise<{ deleted: true } | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${savedSearchPath(id)}`, {
      headers: cookie ? { cookie } : undefined,
      method: "DELETE",
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Saved search could not be deleted.",
      },
      status: 503,
    };
  }

  if (response.status === 204) {
    return { deleted: true };
  }

  const body = await response.json().catch(() => null);
  return (
    (body as ApiErrorEnvelope | null) ?? {
      error: {
        code: "saved_search_delete_failed",
        message: "Saved search could not be deleted.",
      },
      status: response.status,
    }
  );
}

export async function searchGlobalFromCookie(
  cookie: string | null | undefined,
  query: GlobalSearchQuery,
): Promise<ListEnvelope<GlobalSearchResult> | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${globalSearchPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Search is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "search_failed",
          message: "Search failed.",
        },
        status: response.status,
      }
    );
  }

  return body as ListEnvelope<GlobalSearchResult>;
}

export async function searchCodeFromCookie(
  cookie: string | null | undefined,
  query: CodeSearchQuery,
): Promise<CodeSearchResponse | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${codeSearchPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Code search is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "code_search_failed",
          message: "Code search failed.",
        },
        status: response.status,
      }
    );
  }

  return body as CodeSearchResponse;
}

export async function searchCollaborationFromCookie(
  cookie: string | null | undefined,
  query: CollaborationSearchQuery,
): Promise<CollaborationSearchResponse | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${collaborationSearchPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Issue and pull request search is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "collaboration_search_failed",
          message: "Issue and pull request search failed.",
        },
        status: response.status,
      }
    );
  }

  return body as CollaborationSearchResponse;
}

export type DashboardSummaryQuery = {
  feedTab?: DashboardFeedTab;
  eventTypes?: DashboardFeedEventType[];
  repositoryFilter?: string;
};

export function dashboardSummaryPath(
  query: DashboardSummaryQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.feedTab) {
    params.set("feedTab", query.feedTab);
  }
  for (const eventType of query.eventTypes ?? []) {
    params.append("eventType", eventType);
  }
  if (query.repositoryFilter?.trim()) {
    params.set("repositoryFilter", query.repositoryFilter.trim());
  }

  const paramString = params.toString();
  return paramString ? `/api/dashboard?${paramString}` : "/api/dashboard";
}

export async function getDashboardSummaryFromCookie(
  cookie: string | null | undefined,
  query: DashboardSummaryQuery = {},
): Promise<DashboardSummary | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${dashboardSummaryPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as DashboardSummary;
}

export async function saveDashboardFeedPreferences(
  cookie: string | null | undefined,
  preferences: DashboardFeedPreferences,
): Promise<DashboardFeedPreferences> {
  const response = await fetch(
    `${apiBaseUrl()}/api/dashboard/feed-preferences`,
    {
      method: "PUT",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(preferences),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    throw new Error("Dashboard feed preferences failed to save");
  }

  return (await response.json()) as DashboardFeedPreferences;
}

export async function resetDashboardFeedPreferences(
  cookie: string | null | undefined,
): Promise<DashboardFeedPreferences> {
  const response = await fetch(
    `${apiBaseUrl()}/api/dashboard/feed-preferences`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    throw new Error("Dashboard feed preferences failed to reset");
  }

  const body = (await response.json()) as {
    feedPreferences: DashboardFeedPreferences;
  };
  return body.feedPreferences;
}

export function repositoryIssuesPath(
  owner: string,
  repo: string,
  query: RepositoryIssueListQuery = {},
): string {
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
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues${suffix}`;
}

export async function getRepositoryIssuesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryIssueListQuery = {},
): Promise<IssueListView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryIssuesPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Issues are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "issues_failed",
          message: "Issues could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  if (isIssueListView(body)) {
    return body;
  }
  return {
    error: {
      code: "invalid_issues_response",
      message:
        "Issues are temporarily unavailable because the API returned an outdated response shape.",
    },
    status: 502,
    details: {
      reason:
        "Restart the API server so the frontend receives issue filters and metadata.",
    },
  };
}

export function repositoryPullRequestsPath(
  owner: string,
  repo: string,
  query: RepositoryPullRequestListQuery = {},
): string {
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
  if (query.order) {
    params.set("order", query.order);
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls${suffix}`;
}

export async function getRepositoryPullRequestsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryPullRequestListQuery = {},
): Promise<PullRequestListView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestsPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull requests are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "pulls_failed",
          message: "Pull requests could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestListView;
}

export function repositoryActionsDashboardPath(
  owner: string,
  repo: string,
  query: RepositoryActionsDashboardQuery = {},
): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === "") {
      continue;
    }
    params.set(key, String(value));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/actions/dashboard${suffix}`;
}

export function repositoryActionsWorkflowDashboardPath(
  owner: string,
  repo: string,
  workflowFile: string,
  query: RepositoryActionsDashboardQuery = {},
): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (
      key === "workflow" ||
      value === undefined ||
      value === null ||
      value === ""
    ) {
      continue;
    }
    params.set(key, String(value));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/actions/workflows/${encodeURIComponent(workflowFile)}/dashboard${suffix}`;
}

export function repositoryActionsRunDetailPath(
  owner: string,
  repo: string,
  runId: string,
): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/actions/runs/${encodeURIComponent(runId)}/detail`;
}

export function repositoryActionsJobLogDetailPath(
  owner: string,
  repo: string,
  runId: string,
  jobId: string,
  query: RepositoryActionsJobLogDetailQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.q) {
    params.set("q", query.q);
  }
  if (query.selectedMatch) {
    params.set("match", String(query.selectedMatch));
  }
  if (query.timestamps !== undefined && query.timestamps !== null) {
    params.set("timestamps", String(query.timestamps));
  }
  if (query.raw !== undefined && query.raw !== null) {
    params.set("raw", String(query.raw));
  }
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const path = `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/actions/runs/${encodeURIComponent(runId)}/jobs/${encodeURIComponent(
    jobId,
  )}/detail`;
  const queryString = params.toString();
  return queryString ? `${path}?${queryString}` : path;
}

export async function getRepositoryActionsDashboardFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryActionsDashboardQuery = {},
): Promise<RepositoryActionsDashboard | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryActionsDashboardPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Repository Actions are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "actions_dashboard_failed",
          message: "Repository Actions could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryActionsDashboard;
}

export async function getRepositoryActionsWorkflowDashboardFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  workflowFile: string,
  query: RepositoryActionsDashboardQuery = {},
): Promise<RepositoryActionsWorkflowDetail | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryActionsWorkflowDashboardPath(
        owner,
        repo,
        workflowFile,
        query,
      )}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Workflow Actions are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "actions_workflow_failed",
          message: "Workflow Actions could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryActionsWorkflowDetail;
}

export async function getRepositoryActionsRunDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  runId: string,
): Promise<RepositoryActionsRunDetail | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryActionsRunDetailPath(owner, repo, runId)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Workflow run details are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "actions_run_detail_failed",
          message: "Workflow run details could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryActionsRunDetail;
}

export async function getRepositoryActionsJobLogDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  runId: string,
  jobId: string,
  query: RepositoryActionsJobLogDetailQuery = {},
): Promise<RepositoryActionsJobLogDetail | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryActionsJobLogDetailPath(
        owner,
        repo,
        runId,
        jobId,
        query,
      )}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Workflow job logs are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "actions_job_log_detail_failed",
          message: "Workflow job logs could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryActionsJobLogDetail;
}

export function repositoryPullRequestPath(
  owner: string,
  repo: string,
  number: number | string,
): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${encodeURIComponent(String(number))}`;
}

export async function getRepositoryPullRequestFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
): Promise<PullRequestDetailView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull request is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "pull_request_failed",
          message: "Pull request could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestDetailView;
}

export async function getRepositoryPullRequestTimelineFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
): Promise<PullRequestTimelineItem[] | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/timeline`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull request timeline is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "timeline_failed",
          message: "Pull request timeline could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestTimelineItem[];
}

export function repositoryPullRequestFilesPath(
  owner: string,
  repo: string,
  number: number | string,
  query: RepositoryPullRequestDiffQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.view) {
    params.set("view", query.view);
  }
  if (query.whitespace) {
    params.set("whitespace", query.whitespace);
  }
  if (query.commit) {
    params.set("commit", query.commit);
  }
  if (query.filter) {
    params.set("filter", query.filter);
  }
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `${repositoryPullRequestPath(owner, repo, number)}/files${suffix}`;
}

export async function getRepositoryPullRequestFilesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  query: RepositoryPullRequestDiffQuery = {},
): Promise<PullRequestDiffReviewView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestFilesPath(
        owner,
        repo,
        number,
        query,
      )}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull request files are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "pull_request_files_failed",
          message: "Pull request files could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestDiffReviewView;
}

export function repositoryPullRequestComparePath(
  owner: string,
  repo: string,
  base: string,
  head: string,
  options: {
    commits?: number;
    files?: number;
    headOwner?: string;
    headRepo?: string;
  } = {},
): string {
  const params = new URLSearchParams();
  if (options.commits) {
    params.set("commits", String(options.commits));
  }
  if (options.files) {
    params.set("files", String(options.files));
  }
  if (options.headOwner && options.headRepo) {
    params.set("headOwner", options.headOwner);
    params.set("headRepo", options.headRepo);
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head)}${suffix}`;
}

export async function getPullRequestCompareFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  base: string,
  head: string,
  options: {
    commits?: number;
    files?: number;
    headOwner?: string;
    headRepo?: string;
  } = {},
): Promise<PullRequestCompareView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestComparePath(
        owner,
        repo,
        base,
        head,
        options,
      )}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull request comparison is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "compare_failed",
          message: "Pull request comparison could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestCompareView;
}

export async function createPullRequestFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: CreatePullRequestRequest,
): Promise<CreatedPullRequest> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
      repo,
    )}/pulls`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Pull request could not be created.",
      { cause: envelope },
    );
  }

  return payload as CreatedPullRequest;
}

export async function createRepositoryPullRequestCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  body: string,
): Promise<PullRequestTimelineItem> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/comments`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ body }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Comment could not be posted", {
      cause: envelope,
    });
  }

  return (await response.json()) as PullRequestTimelineItem;
}

export async function updateRepositoryPullRequestReviewRequestsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  reviewerUserIds: string[],
): Promise<PullRequestDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/review-requests`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ reviewerUserIds }),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review requests could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDetailView;
}

export async function updateRepositoryPullRequestDraftFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  isDraft: boolean,
): Promise<PullRequestDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/draft`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ isDraft }),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Draft state could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDetailView;
}

export async function updateRepositoryPullRequestStateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  state: PullRequestState,
): Promise<PullRequestDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ state }),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Pull request state could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDetailView;
}

export async function mergeRepositoryPullRequestFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  request: {
    method: MergeMethod;
    commitTitle?: string | null;
    commitBody?: string | null;
    deleteBranch?: boolean;
  },
): Promise<PullRequestDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/merge`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Pull request could not merge", {
      cause: envelope,
    });
  }
  return (await response.json()) as PullRequestDetailView;
}

export async function updateRepositoryPullRequestMetadataFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  request: UpdatePullRequestMetadataRequest,
): Promise<PullRequestDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/metadata`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Pull request metadata could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDetailView;
}

export async function updateRepositoryPullRequestSubscriptionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  subscribed: boolean,
): Promise<PullRequestSubscriptionState> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/subscription`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ subscribed }),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Notification subscription could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestSubscriptionState;
}

export async function updateRepositoryPullRequestViewedFileFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  request: {
    fileId: string;
    versionKey: string;
    viewed: boolean;
  },
): Promise<PullRequestViewedFileState> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/files/viewed`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Viewed file state could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestViewedFileState;
}

export async function createRepositoryPullRequestReviewDraftCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  request: CreatePullRequestReviewDraftCommentRequest,
): Promise<PullRequestDiffReviewComment> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/review-comments/drafts`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review comment draft could not be saved",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDiffReviewComment;
}

export async function updateRepositoryPullRequestReviewDraftCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  draftId: string,
  request: UpdatePullRequestReviewDraftCommentRequest,
): Promise<PullRequestDiffReviewComment> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(
      owner,
      repo,
      number,
    )}/review-comments/drafts/${encodeURIComponent(draftId)}`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review comment draft could not be updated",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDiffReviewComment;
}

export async function deleteRepositoryPullRequestReviewDraftCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  draftId: string,
): Promise<PullRequestDiffPendingReview> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(
      owner,
      repo,
      number,
    )}/review-comments/drafts/${encodeURIComponent(draftId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review comment draft could not be deleted",
      { cause: envelope },
    );
  }
  return (await response.json()) as PullRequestDiffPendingReview;
}

export async function submitRepositoryPullRequestReviewFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
  request: SubmitPullRequestReviewRequest,
): Promise<PullRequestSubmittedReview> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/reviews`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review could not be submitted",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as PullRequestSubmittedReview;
}

export async function abandonRepositoryPullRequestReviewDraftFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
): Promise<PullRequestDiffPendingReview> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(
      owner,
      repo,
      number,
    )}/reviews/draft`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Review draft could not be abandoned",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as PullRequestDiffPendingReview;
}

export async function saveRepositoryPullPreferences(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  preferences: Pick<PullRequestListPreferences, "dismissedContributorBanner">,
): Promise<PullRequestListPreferences> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
      repo,
    )}/pulls/preferences`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(preferences),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    throw new Error("Pull request preferences failed to save");
  }

  return (await response.json()) as PullRequestListPreferences;
}

export function repositoryIssuePath(
  owner: string,
  repo: string,
  issueNumber: number | string,
): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${encodeURIComponent(String(issueNumber))}`;
}

export async function getRepositoryIssueFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
): Promise<IssueDetailView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Issue is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "issue_failed",
          message: "Issue could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as IssueDetailView;
}

export async function getRepositoryIssueTimelineFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
): Promise<IssueTimelineItem[] | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/timeline`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Issue timeline is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "timeline_failed",
          message: "Issue timeline could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as IssueTimelineItem[];
}

export async function createRepositoryIssueCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  body: string,
): Promise<IssueTimelineItem> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/comments`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ body }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Comment could not be posted", {
      cause: envelope,
    });
  }

  return (await response.json()) as IssueTimelineItem;
}

export async function updateRepositoryIssueStateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  state: IssueState,
): Promise<IssueDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ state }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Issue state could not be updated",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as IssueDetailView;
}

export async function toggleRepositoryIssueReactionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  content: ReactionContent,
): Promise<ReactionSummary[]> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/reactions`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ content }),
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Reaction could not be updated",
      {
        cause: envelope,
      },
    );
  }

  return ((payload as { summaries?: ReactionSummary[] } | null)?.summaries ??
    []) as ReactionSummary[];
}

export async function updateRepositoryIssueSubscriptionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  subscribed: boolean,
): Promise<IssueSubscriptionState> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/subscription`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ subscribed }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Notification subscription could not be updated",
      { cause: envelope },
    );
  }

  return (await response.json()) as IssueSubscriptionState;
}

export async function updateRepositoryIssueMetadataFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  request: UpdateIssueMetadataRequest,
): Promise<IssueDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/metadata`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Issue metadata could not be updated",
      { cause: envelope },
    );
  }

  return (await response.json()) as IssueDetailView;
}

export async function saveRepositoryIssuePreferences(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  preferences: Pick<IssueListPreferences, "dismissedContributorBanner">,
): Promise<IssueListPreferences> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
      repo,
    )}/issues/preferences`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(preferences),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    throw new Error("Issue preferences failed to save");
  }

  return (await response.json()) as IssueListPreferences;
}

export async function createRepositoryIssueFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: CreateIssueRequest,
): Promise<CreatedIssue> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Issue could not be created", {
      cause: body,
    });
  }

  return (await response.json()) as CreatedIssue;
}

export async function getRepositoryIssueTemplatesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<IssueTemplate[]> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/templates`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    return [];
  }

  const body = (await response.json()) as IssueTemplateList;
  return body.items;
}

export async function getRepositoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryOverview | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURI(owner)}/${encodeURI(repo)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryOverview;
}

export async function getRepositorySettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositorySettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Repository settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository settings are unavailable right now.";
    try {
      const body = (await response.json()) as {
        error?: { code?: string; message?: string };
      };
      code = body.error?.code ?? null;
      message = body.error?.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    settings: (await response.json()) as RepositorySettings,
  };
}

export async function updateRepositorySettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  patch: RepositorySettingsPatch,
): Promise<RepositorySettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(patch),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      body?.error.message ?? "Repository settings failed to save",
      {
        cause: body,
      },
    );
  }

  return (await response.json()) as RepositorySettings;
}

export async function getRepositoryPathFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  refName: string,
  path: string,
  options: { page?: number; pageSize?: number } = {},
): Promise<RepositoryPathOverview | null> {
  const normalizedPath = path.replace(/^\/+|\/+$/g, "");
  const encodedPath = normalizedPath
    .split("/")
    .filter(Boolean)
    .map(encodeURIComponent)
    .join("/");
  const params = new URLSearchParams({ ref: refName });
  if (options.page) {
    params.set("page", String(options.page));
  }
  if (options.pageSize) {
    params.set("pageSize", String(options.pageSize));
  }
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${encodedPath}?${params.toString()}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryPathOverview;
}

export async function getRepositoryBlobFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  refName: string,
  path: string,
): Promise<RepositoryBlobView | null> {
  const encodedPath = path
    .replace(/^\/+|\/+$/g, "")
    .split("/")
    .filter(Boolean)
    .map(encodeURIComponent)
    .join("/");
  const params = new URLSearchParams({ ref: refName });
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/blobs/${encodedPath}?${params.toString()}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryBlobView;
}

export async function getRepositoryBlameFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  refName: string,
  path: string,
): Promise<RepositoryBlameView | null> {
  const encodedPath = path
    .replace(/^\/+|\/+$/g, "")
    .split("/")
    .filter(Boolean)
    .map(encodeURIComponent)
    .join("/");
  const params = new URLSearchParams({ ref: refName });
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/blame/${encodedPath}?${params.toString()}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryBlameView;
}

export async function getRepositoryCommitHistoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  refName: string,
  path = "",
): Promise<ListEnvelope<RepositoryCommitHistoryItem> | null> {
  const params = new URLSearchParams({ ref: refName });
  const normalizedPath = path.replace(/^\/+|\/+$/g, "");
  if (normalizedPath) {
    params.set("path", normalizedPath);
  }
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/commits?${params.toString()}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as ListEnvelope<RepositoryCommitHistoryItem>;
}

export async function getRepositoryRefsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: {
    query?: string;
    currentPath?: string;
    activeRef?: string;
    page?: number;
    pageSize?: number;
  } = {},
): Promise<ListEnvelope<RepositoryRefSummary> | null> {
  const params = new URLSearchParams();
  if (options.query?.trim()) {
    params.set("q", options.query.trim());
  }
  if (options.currentPath?.trim()) {
    params.set("currentPath", options.currentPath.trim());
  }
  if (options.activeRef?.trim()) {
    params.set("activeRef", options.activeRef.trim());
  }
  if (options.page) {
    params.set("page", String(options.page));
  }
  if (options.pageSize) {
    params.set("pageSize", String(options.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/refs${suffix}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as ListEnvelope<RepositoryRefSummary>;
}

export async function getRepositoryFileFinderFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  refName: string,
  query: string,
  options: { page?: number; pageSize?: number } = {},
): Promise<RepositoryFileFinderResult | null> {
  const params = new URLSearchParams({ ref: refName });
  if (query.trim()) {
    params.set("q", query.trim());
  }
  if (options.page) {
    params.set("page", String(options.page));
  }
  if (options.pageSize) {
    params.set("pageSize", String(options.pageSize));
  }
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/file-finder?${params.toString()}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryFileFinderResult;
}

export async function setRepositoryStarFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  starred: boolean,
): Promise<RepositorySocialState> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/star`,
    {
      method: starred ? "PUT" : "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository star update failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositorySocialState;
}

export async function setRepositoryWatchFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  watching: boolean,
): Promise<RepositorySocialState> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/watch`,
    {
      method: watching ? "PUT" : "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository watch update failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositorySocialState;
}

export async function forkRepositoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryForkResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/forks`,
    {
      method: "POST",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository fork failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositoryForkResult;
}

export async function getRepositoryCreationOptionsFromCookie(
  cookie: string | null | undefined,
): Promise<RepositoryCreationOptions | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/repos/creation-options`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryCreationOptions;
}

export function repositoryNameAvailabilityPath({
  ownerType,
  ownerId,
  name,
}: {
  ownerType: RepositoryOwnerType;
  ownerId: string;
  name: string;
}): string {
  const params = new URLSearchParams({
    ownerType,
    ownerId,
    name,
  });
  return `/api/repos/name-availability?${params.toString()}`;
}

export async function getRepositoryNameAvailabilityFromCookie(
  cookie: string | null | undefined,
  query: {
    ownerType: RepositoryOwnerType;
    ownerId: string;
    name: string;
  },
): Promise<RepositoryNameAvailability | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryNameAvailabilityPath(query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryNameAvailability;
}

export async function createRepositoryFromCookie(
  cookie: string | null | undefined,
  request: CreateRepositoryRequest,
): Promise<CreatedRepository> {
  const response = await fetch(`${apiBaseUrl()}/api/repos`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(request),
    cache: "no-store",
  });

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository could not be created", {
      cause: body,
    });
  }

  return (await response.json()) as CreatedRepository;
}

export async function createRepositoryImportFromCookie(
  cookie: string | null | undefined,
  request: RepositoryImportRequest,
): Promise<RepositoryImportStatus> {
  const response = await fetch(`${apiBaseUrl()}/api/repos/imports`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(request),
    cache: "no-store",
  });

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      body?.error.message ?? "Repository import could not start",
      {
        cause: body,
      },
    );
  }

  return (await response.json()) as RepositoryImportStatus;
}

export async function getRepositoryImportFromCookie(
  cookie: string | null | undefined,
  importId: string,
): Promise<RepositoryImportStatus | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/repos/imports/${importId}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as RepositoryImportStatus;
}

export async function logout(cookie: string | null): Promise<string | null> {
  const response = await fetch(`${apiBaseUrl()}/api/auth/logout`, {
    method: "POST",
    headers: cookie ? { cookie } : undefined,
    cache: "no-store",
  });

  return response.headers.get("set-cookie");
}

export async function renderMarkdown(
  request: RenderMarkdownRequest,
): Promise<RenderedMarkdown> {
  const response = await fetch(`${apiBaseUrl()}/api/markdown/render`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(request),
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error("Markdown preview failed");
  }

  return (await response.json()) as RenderedMarkdown;
}

export async function highlightCode(
  request: HighlightCodeRequest,
): Promise<HighlightedFile> {
  const response = await fetch(`${apiBaseUrl()}/api/highlight/render`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(request),
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error("Syntax highlighting failed");
  }

  return (await response.json()) as HighlightedFile;
}

export type NotificationInboxQueryView = {
  q: string;
  folder: string;
  tab: string;
  sort: string;
  group: string;
  repo: string | null;
};

export type NotificationFacet = {
  id: string;
  label: string;
  query: string;
  href: string;
  count: number;
  active: boolean;
};

export type NotificationChoice = {
  id: string;
  label: string;
  href: string;
  active: boolean;
};

export type NotificationInboxRow = {
  id: string;
  repositoryId: string | null;
  repositoryName: string;
  repositoryHref: string | null;
  subjectType: string;
  subjectNumber: number | null;
  title: string;
  reason: string;
  reasonLabel: string;
  href: string;
  openHref: string;
  unread: boolean;
  saved: boolean;
  done: boolean;
  subscribed: boolean;
  updatedAt: string;
  relativeTime: string;
};

export type NotificationGroup = {
  id: string;
  label: string;
  count: number;
  rows: NotificationInboxRow[];
};

export type NotificationInboxView = {
  query: NotificationInboxQueryView;
  folders: NotificationFacet[];
  filters: NotificationFacet[];
  repositories: NotificationFacet[];
  sortOptions: NotificationChoice[];
  groupOptions: NotificationChoice[];
  groups: NotificationGroup[];
  total: number;
  unreadCount: number;
  page: number;
  pageSize: number;
  emptyTitle: string;
  emptyMessage: string;
};

export type NotificationInboxQuery = {
  q?: string;
  folder?: string;
  tab?: string;
  sort?: string;
  group?: string;
  repo?: string;
  page?: string;
  pageSize?: string;
};

export function notificationsPath(query: NotificationInboxQuery = {}): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value?.trim()) {
      params.set(key, value.trim());
    }
  }
  const suffix = params.toString();
  return suffix ? `/api/notifications?${suffix}` : "/api/notifications";
}

export async function getNotificationsFromCookie(
  cookie: string | null | undefined,
  query: NotificationInboxQuery = {},
): Promise<NotificationInboxView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${notificationsPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Notifications are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "notifications_failed",
          message: "Notifications could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as NotificationInboxView;
}

export async function markNotificationReadFromCookie(
  cookie: string | null | undefined,
  notificationId: string,
): Promise<boolean> {
  try {
    const response = await fetch(
      `${apiBaseUrl()}/api/notifications/${encodeURIComponent(notificationId)}/read`,
      {
        method: "PATCH",
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
    return response.ok;
  } catch {
    return false;
  }
}
