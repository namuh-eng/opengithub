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

export type ApiUser = {
  id: string;
  login: string;
  name: string | null;
  email: string;
  avatarUrl: string | null;
  htmlUrl: string;
  createdAt: string;
  updatedAt: string;
};

export type GistOwner = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  href: string;
};

export type GistFile = {
  id: string;
  filename: string;
  language: string | null;
  sizeBytes: number;
  contentSha: string;
  content: string;
  position: number;
};

export type GistSummary = {
  id: string;
  description: string | null;
  isPublic: boolean;
  owner: GistOwner;
  files: GistFile[];
  commentsCount: number;
  starsCount: number;
  forksCount: number;
  cloneUrl: string;
  embedUrl: string;
  href: string;
  createdAt: string;
  updatedAt: string;
};

export type GistDetail = GistSummary & {
  comments: Array<{
    id: string;
    body: string;
    author: GistOwner;
    createdAt: string;
    updatedAt: string;
  }>;
  viewer: {
    authenticated: boolean;
    canEdit: boolean;
    isStarred: boolean;
  };
};

export type GistRevisionList = {
  gist: GistSummary;
  revisions: Array<{
    id: string;
    version: number;
    description: string | null;
    files: GistFile[];
    author: GistOwner;
    createdAt: string;
  }>;
};

export type GistList = {
  items: GistSummary[];
  total: number;
  page: number;
  pageSize: number;
  scope: string;
};

export type GistMutationRequest = {
  description?: string | null;
  isPublic?: boolean;
  files: Array<{ filename: string; content: string }>;
};

export type RepositoryLabelCounts = {
  openIssues: number;
  openPullRequests: number;
  discussions: number;
  totalIssueCount: number;
};

export type RepositoryLabelSummary = {
  id: string;
  name: string;
  color: string;
  description: string | null;
  isDefault: boolean;
  counts: RepositoryLabelCounts;
  issuesHref: string;
  pullRequestsHref: string;
  discussionsHref: string;
  createdAt: string;
  updatedAt: string;
};

export type RepositoryLabelsView = {
  items: RepositoryLabelSummary[];
  total: number;
  page: number;
  pageSize: number;
  filters: {
    query: string | null;
    sort: "name" | "total_issue_count" | string;
    direction: "asc" | "desc" | string;
  };
  viewer: {
    authenticated: boolean;
    role: string | null;
    canRead: boolean;
    canWrite: boolean;
    canAdmin: boolean;
  };
  repository: {
    id: string;
    owner: string;
    name: string;
    visibility: RepositoryVisibility | string;
    isArchived: boolean;
  };
};

export type RepositoryLabelsQuery = {
  q?: string | null;
  sort?: "name" | "total_issue_count" | string | null;
  direction?: "asc" | "desc" | string | null;
  page?: number | null;
  pageSize?: number | null;
};

export type RepositoryLabelMutationRequest = {
  name: string;
  color: string;
  description?: string | null;
};

export type RepositoryLabelMutationResult = {
  label: RepositoryLabelSummary;
  eventId: string;
};

export type PersonalAccessTokenResourceOwner = {
  id: string;
  kind: "user" | "organization" | string;
  login: string;
  displayName: string;
  avatarUrl: string | null;
};

export type PersonalAccessTokenRepositorySummary = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  visibility: RepositoryVisibility | string;
};

export type PersonalAccessTokenSummary = {
  id: string;
  name: string;
  description: string;
  type: "fine_grained" | "classic" | string;
  prefix: string;
  scopes: string[];
  resourceOwner: PersonalAccessTokenResourceOwner;
  repositoryAccess: "all" | "selected" | string;
  selectedRepositories: PersonalAccessTokenRepositorySummary[];
  status: "active" | "expired" | "revoked" | string;
  lastUsedAt: string | null;
  expiresAt: string | null;
  revokedAt: string | null;
  createdAt: string;
};

export type PersonalAccessTokenSudoState = {
  active: boolean;
  expiresAt: string | null;
  requiredFor: string[];
};

export type AccountSignInMethod = {
  id: string;
  provider: string;
  email: string;
  displayLabel: string;
  avatarUrl: string | null;
  linkedAt: string;
  updatedAt: string;
  canUnlink: boolean;
};

export type AccountSecuritySettings = {
  signInMethods: AccountSignInMethod[];
  sudo: PersonalAccessTokenSudoState;
  twoFactor: {
    enabled: boolean;
    available: boolean;
    reason: string;
  };
};

export type AccountSecuritySettingsFetchResult =
  | { ok: true; settings: AccountSecuritySettings }
  | { ok: false; status: number; code: string | null; message: string };

export type AccountSessionSummary = {
  id: string;
  device: string;
  browser: string;
  location: string;
  ipAddress: string | null;
  userAgent: string | null;
  signedInAt: string;
  lastActiveAt: string;
  expiresAt: string;
  isCurrent: boolean;
};

export type AccountSessions = {
  sessions: AccountSessionSummary[];
  activeCount: number;
  currentSessionId: string;
};

export type AccountSessionsFetchResult =
  | { ok: true; sessions: AccountSessions }
  | { ok: false; status: number; code: string | null; message: string };

export type AccountSecurityLogEvent = {
  id: string;
  action: string;
  location: string;
  ipAddress: string | null;
  userAgent: string | null;
  metadata: Record<string, unknown>;
  createdAt: string;
};

export type AccountSecurityLog = {
  events: AccountSecurityLogEvent[];
  actions: string[];
  filters: {
    action: string | null;
    page: number;
    pageSize: number;
  };
  pagination: {
    total: number;
    page: number;
    pageSize: number;
    totalPages: number;
    hasPrevious: boolean;
    hasNext: boolean;
  };
};

export type AccountSecurityLogFetchResult =
  | { ok: true; log: AccountSecurityLog }
  | { ok: false; status: number; code: string | null; message: string };

export type AccountSecurityLogQuery = {
  action?: string | null;
  page?: number | null;
};

export type ProjectListScopeSummary = {
  kind: "user" | "organization" | "repository" | string;
  login: string;
  repository: ProjectRepositoryScopeSummary | null;
  href: string;
};

export type ProjectRepositoryScopeSummary = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  href: string;
};

export type ProjectStatusSummary = {
  status: "on_track" | "at_risk" | "off_track" | "complete" | string;
  label: string;
  body: string | null;
  createdAt: string;
};

export type ProjectItemCounts = {
  total: number;
  open: number;
  closed: number;
  draft: number;
};

export type ProjectRow = {
  id: string;
  number: number;
  title: string;
  description: string | null;
  state: "open" | "closed" | string;
  visibility: "public" | "private" | string;
  href: string;
  workspaceHref: string;
  owner: string;
  isTemplate: boolean;
  defaultRepository: ProjectRepositoryScopeSummary | null;
  linkedRepositoriesCount: number;
  status: ProjectStatusSummary | null;
  counts: ProjectItemCounts;
  viewerRole: string | null;
  viewerCanCopy: boolean;
  createdAt: string;
  updatedAt: string;
  closedAt: string | null;
};

export type ProjectTemplateRow = {
  id: string;
  projectId: string;
  title: string;
  description: string | null;
  projectTitle: string;
  projectHref: string;
  isPublic: boolean;
  viewerCanCopy: boolean;
  createdAt: string;
};

export type ProjectCounts = {
  open: number;
  closed: number;
  templates: number;
  total: number;
};

export type ProjectListFilters = {
  query: string | null;
  state: "open" | "closed" | string;
  tab: "projects" | "templates" | string;
  sort:
    | "recently_updated"
    | "name_asc"
    | "name_desc"
    | "created_asc"
    | "created_desc"
    | string;
  page: number;
  pageSize: number;
};

export type ProjectListPermissions = {
  authenticated: boolean;
  viewerRole: string | null;
  canCreate: boolean;
  canCopy: boolean;
};

export type ProjectList = {
  items: ProjectRow[];
  total: number;
  page: number;
  pageSize: number;
  scope: ProjectListScopeSummary;
  filters: ProjectListFilters;
  counts: ProjectCounts;
  templates: {
    items: ProjectTemplateRow[];
    total: number;
    page: number;
    pageSize: number;
  };
  viewerPermissions: ProjectListPermissions;
  unavailableReason: string | null;
};

export type ProjectListQuery = {
  q?: string | null;
  state?: "open" | "closed" | string | null;
  tab?: "projects" | "templates" | string | null;
  sort?: string | null;
  page?: number | null;
  pageSize?: number | null;
};

export type ProjectListFetchResult =
  | { ok: true; projects: ProjectList }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectWorkspaceProject = {
  id: string;
  number: number;
  title: string;
  description: string | null;
  state: "open" | "closed" | string;
  visibility: "public" | "private" | string;
  owner: string;
  href: string;
  workspaceHref: string;
  viewerRole: string | null;
};

export type ProjectWorkspaceView = {
  id: string;
  number: number;
  name: string;
  layout: "table" | "board" | "roadmap" | string;
  href: string;
  configuration: Record<string, unknown>;
  updatedAt: string;
};

export type ProjectWorkspaceLayoutChoice = {
  layout: "table" | "board" | "roadmap" | string;
  label: string;
  keyboardHint: string;
  active: boolean;
  enabled: boolean;
  unavailableReason: string | null;
};

export type ProjectWorkspaceField = {
  id: string;
  name: string;
  fieldType: string;
  position: number;
  settings: Record<string, unknown>;
  hidden: boolean;
  editable: boolean;
};

export type ProjectWorkspaceLayoutField = {
  id: string;
  name: string;
  fieldType: string;
};

export type ProjectWorkspaceBoardColumn = {
  key: string;
  label: string;
  fieldId: string;
  count: number;
  itemLimit: number | null;
  overLimit: boolean;
  visible: boolean;
};

export type ProjectWorkspaceBoardConfig = {
  columnField: ProjectWorkspaceLayoutField | null;
  swimlaneField: ProjectWorkspaceLayoutField | null;
  eligibleColumnFields: ProjectWorkspaceLayoutField[];
  eligibleSwimlaneFields: ProjectWorkspaceLayoutField[];
  columns: ProjectWorkspaceBoardColumn[];
  emptyColumnsVisible: boolean;
  unavailableReason: string | null;
};

export type ProjectWorkspaceRoadmapConfig = {
  startDateField: ProjectWorkspaceLayoutField | null;
  targetDateField: ProjectWorkspaceLayoutField | null;
  markerFields: ProjectWorkspaceLayoutField[];
  eligibleDateFields: ProjectWorkspaceLayoutField[];
  eligibleMarkerFields: ProjectWorkspaceLayoutField[];
  zoom: "month" | "quarter" | "year" | string;
  zoomOptions: string[];
  unavailableReason: string | null;
};

export type ProjectWorkspaceFieldValue = {
  fieldId: string;
  value: unknown;
  displayValue: string;
};

export type ProjectWorkspaceLabel = {
  id: string;
  name: string;
  color: string;
};

export type ProjectWorkspaceUser = {
  id: string;
  login: string;
  avatarUrl: string | null;
};

export type ProjectWorkspaceItem = {
  id: string;
  itemType: "draft_issue" | "issue" | "pull_request" | string;
  position: string;
  title: string;
  body: string | null;
  state: string | null;
  number: number | null;
  href: string | null;
  repository: ProjectRepositoryScopeSummary | null;
  fieldValues: ProjectWorkspaceFieldValue[];
  labels: ProjectWorkspaceLabel[];
  assignees: ProjectWorkspaceUser[];
  updatedAt: string;
};

export type ProjectWorkspaceGroup = {
  key: string;
  label: string;
  count: number;
};

export type ProjectWorkspaceSlice = ProjectWorkspaceGroup;

export type ProjectWorkspaceFilters = {
  query: string | null;
  sort: string;
  group: string | null;
  slice: string | null;
  tokens: string[];
  page: number;
  pageSize: number;
};

export type ProjectWorkspace = {
  project: ProjectWorkspaceProject;
  selectedView: ProjectWorkspaceView;
  views: ProjectWorkspaceView[];
  layoutChoices?: ProjectWorkspaceLayoutChoice[];
  fields: ProjectWorkspaceField[];
  boardConfig?: ProjectWorkspaceBoardConfig | null;
  roadmapConfig?: ProjectWorkspaceRoadmapConfig | null;
  items: ProjectWorkspaceItem[];
  total: number;
  page: number;
  pageSize: number;
  groups: ProjectWorkspaceGroup[];
  slices: ProjectWorkspaceSlice[];
  filters: ProjectWorkspaceFilters;
  unsavedView: {
    active: boolean;
    reasons: string[];
  };
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canEdit: boolean;
    canManageViews: boolean;
    canChangeLayout?: boolean;
    canAddItems: boolean;
  };
  unavailableReason: string | null;
};

export type ProjectItemSourceSummary = {
  sourceType: string;
  id: string;
  number: number;
  title: string;
  state: string;
  href: string;
  repository: ProjectRepositoryScopeSummary;
  updatedAt: string;
  syncedAt: string | null;
  syncVersion: number;
};

export type ProjectItemActivity = {
  id: string;
  eventType: string;
  actor: ProjectWorkspaceUser | null;
  metadata: unknown;
  createdAt: string;
};

export type ProjectItemComment = {
  id: string;
  author: ProjectWorkspaceUser;
  body: string;
  isDeleted: boolean;
  createdAt: string;
  updatedAt: string;
};

export type ProjectItemArchiveState = {
  archived: boolean;
  archivedAt: string | null;
  archivedBy: ProjectWorkspaceUser | null;
  restoredAt: string | null;
  restoredBy: ProjectWorkspaceUser | null;
};

export type ProjectDraftIssueMetadata = {
  editable: boolean;
  editVersion: string;
  repositoryNotificationsEnabled: boolean;
};

export type ProjectItemDetailPermissions = {
  authenticated: boolean;
  viewerRole: string | null;
  canEdit: boolean;
  canComment: boolean;
  canConvert: boolean;
  canArchive: boolean;
  canRestore: boolean;
  canRemove: boolean;
};

export type ProjectItemDetail = {
  project: ProjectWorkspaceProject;
  item: ProjectWorkspaceItem;
  source: ProjectItemSourceSummary | null;
  activity: ProjectItemActivity[];
  comments: ProjectItemComment[];
  archive: ProjectItemArchiveState;
  draft: ProjectDraftIssueMetadata | null;
  viewerPermissions: ProjectItemDetailPermissions;
  unavailableReason: string | null;
};

export type ProjectDraftUpdateRequest = {
  title: string;
  body: string | null;
  expectedUpdatedAt: string | null;
};

export type ProjectItemCommentCreateRequest = {
  body: string;
  expectedUpdatedAt: string | null;
};

export type ProjectItemCommentUpdateRequest = {
  body: string;
  expectedUpdatedAt: string | null;
};

export type ProjectConversionMilestone = {
  id: string;
  title: string;
  state: string;
};

export type ProjectConversionRepository = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  href: string;
  labels: ProjectWorkspaceLabel[];
  assignees: ProjectWorkspaceUser[];
  milestones: ProjectConversionMilestone[];
};

export type ProjectConversionTargets = {
  project: ProjectWorkspaceProject;
  repositories: ProjectConversionRepository[];
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canConvert: boolean;
  };
};

export type ProjectDraftConvertRequest = {
  repositoryId: string;
  labelIds: string[];
  assigneeUserIds: string[];
  milestoneId: string | null;
  expectedUpdatedAt: string | null;
};

export type ProjectArchivedItem = {
  item: ProjectWorkspaceItem;
  source: ProjectItemSourceSummary | null;
  archivedAt: string;
  archivedBy: ProjectWorkspaceUser | null;
  viewerPermissions: ProjectItemDetailPermissions;
};

export type ProjectArchivedItems = {
  items: ProjectArchivedItem[];
  total: number;
  page: number;
  pageSize: number;
};

export type ProjectItemDetailFetchResult =
  | { ok: true; detail: ProjectItemDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectArchivedItemsFetchResult =
  | { ok: true; archived: ProjectArchivedItems }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectArchivedItemsQuery = {
  itemType?: "draft_issue" | "issue" | "pull_request" | "all" | null;
  q?: string | null;
  page?: number | null;
  pageSize?: number | null;
};

export type ProjectInsightsQuery = {
  chart?: string | null;
  range?: "2w" | "1m" | "3m" | "max" | "custom" | string | null;
  start?: string | null;
  end?: string | null;
  filter?: string | null;
  table?: boolean | null;
};

export type ProjectInsightsChartSummary = {
  id: string;
  title: string;
  description: string | null;
  chartType: string;
  href: string;
  shareHref: string;
  visibility: string;
  sharedWithViewers: boolean;
  updatedAt: string;
};

export type ProjectInsightsChart = ProjectInsightsChartSummary & {
  isDefault: boolean;
  configuration: Record<string, unknown>;
};

export type ProjectInsightsRange = {
  key: string;
  label: string;
  start: string;
  end: string;
  options: Array<{
    key: string;
    label: string;
    href: string;
    active: boolean;
  }>;
};

export type ProjectInsightsFilter = {
  query: string | null;
  tokens: string[];
  unsupportedTokens: string[];
};

export type ProjectInsightsSeries = {
  id: string;
  name: string;
  color: string;
  points: Array<{
    date: string;
    value: number;
  }>;
};

export type ProjectInsightsDataRow = {
  itemId: string;
  itemType: string;
  title: string;
  state: string | null;
  repository: ProjectRepositoryScopeSummary | null;
  createdAt: string;
  completedAt: string | null;
};

export type ProjectInsights = {
  project: ProjectWorkspaceProject;
  navigation: {
    returnHref: string;
    insightsHref: string;
    selectedItem: string;
  };
  selectedChart: ProjectInsightsChart;
  defaultCharts: ProjectInsightsChartSummary[];
  customCharts: ProjectInsightsChartSummary[];
  range: ProjectInsightsRange;
  filter: ProjectInsightsFilter;
  matchingItemCount: number;
  series: ProjectInsightsSeries[];
  dataRows: ProjectInsightsDataRow[];
  latestStatus: ProjectStatusSummary | null;
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canViewInsights: boolean;
    canCreateCharts: boolean;
    canEditCharts: boolean;
    canDeleteCharts: boolean;
    canShareCharts: boolean;
    canViewStatus: boolean;
  };
  cache: {
    cacheKey: string;
    computedAt: string;
    stale: boolean;
    version: number;
  };
  unavailableReason: string | null;
};

export type ProjectInsightsFetchResult =
  | { ok: true; insights: ProjectInsights }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectInsightsChartMutationRequest = {
  title: string;
  description?: string | null;
  chartType: "burn_up" | "bar" | "line" | "stacked_area" | "number" | string;
  filter?: string | null;
  xFieldId?: string | null;
  yFieldId?: string | null;
  groupFieldId?: string | null;
  visibility: "private" | "project" | string;
  expectedUpdatedAt?: string | null;
};

export type ProjectWorkspaceQuery = {
  view?: string | number | null;
  q?: string | null;
  sort?: string | null;
  group?: string | null;
  slice?: string | null;
  page?: number | null;
  pageSize?: number | null;
};

export type ProjectWorkspaceFetchResult =
  | { ok: true; workspace: ProjectWorkspace }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectFieldOption = {
  id: string;
  name: string;
  color: string;
  position: number;
  description: string | null;
};

export type ProjectIteration = {
  id: string;
  name: string;
  startDate: string;
  durationDays: number;
  position: number;
};

export type ProjectIterationBreak = {
  id: string;
  name: string;
  startDate: string;
  durationDays: number;
};

export type ProjectFieldSettingsField = {
  id: string;
  name: string;
  fieldType: string;
  position: number;
  settings: Record<string, unknown>;
  builtIn: boolean;
  editable: boolean;
  deletable: boolean;
  usageCount: number;
  options: ProjectFieldOption[];
  iterations: ProjectIteration[];
  breaks: ProjectIterationBreak[];
  cacheVersion: number;
  updatedAt: string;
};

export type ProjectFieldSettings = {
  project: ProjectWorkspaceProject;
  fields: ProjectFieldSettingsField[];
  limits: {
    maxFields: number;
    usedFields: number;
    remainingFields: number;
    maxOptionsPerField: number;
    maxIterationsPerField: number;
  };
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canCreateFields: boolean;
    canRenameFields: boolean;
    canDeleteFields: boolean;
    canManageOptions: boolean;
    canManageIterations: boolean;
  };
  unavailableReason: string | null;
};

export type ProjectFieldSettingsFetchResult =
  | { ok: true; settings: ProjectFieldSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectWorkflowRule = {
  id: string;
  ruleType: string;
  configuration: Record<string, unknown>;
  position: number;
};

export type ProjectWorkflowDefinition = {
  id: string;
  workflowKey: string;
  name: string;
  description: string;
  enabled: boolean;
  triggerEvent: string;
  configuration: Record<string, unknown>;
  rules: ProjectWorkflowRule[];
  repositoryTargetIds: string[];
  actorLabel: string;
  source: "system" | "ui" | "actions" | "graphql" | string;
  lastRunAt: string | null;
  lastRunStatus: "success" | "skipped" | "failed" | string | null;
  lastRunMessage: string | null;
  updatedAt: string;
};

export type ProjectWorkflowEligibleField = {
  id: string;
  name: string;
  fieldType: string;
  options: ProjectFieldOption[];
  supportsStatusTarget: boolean;
  supportsArchiveCriteria: boolean;
};

export type ProjectWorkflowRepositoryTarget = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  href: string;
  visibility: string;
  permission: string;
};

export type ProjectWorkflowExecutionLog = {
  id: string;
  workflowId: string | null;
  workflowKey: string | null;
  itemId: string | null;
  actor: ProjectWorkspaceUser | null;
  source: "system" | "ui" | "actions" | "graphql" | string;
  eventType: string;
  status: "success" | "skipped" | "failed" | string;
  message: string | null;
  metadata: Record<string, unknown>;
  createdAt: string;
};

export type ProjectWorkflowSettings = {
  project: ProjectWorkspaceProject;
  workflows: ProjectWorkflowDefinition[];
  eligibleFields: ProjectWorkflowEligibleField[];
  repositoryTargets: ProjectWorkflowRepositoryTarget[];
  recentLogs: ProjectWorkflowExecutionLog[];
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canManageWorkflows: boolean;
    canViewLogs: boolean;
    canSelectRepositories: boolean;
  };
  automationActor: string;
  unavailableReason: string | null;
};

export type ProjectWorkflowSettingsFetchResult =
  | { ok: true; settings: ProjectWorkflowSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectWorkflowUpdateRequest = {
  enabled?: boolean;
  condition?: string;
  statusFieldId?: string | null;
  statusOptionId?: string | null;
  repositoryTargetIds?: string[];
  archiveAfterDays?: number | null;
  closeOnStatus?: boolean;
  expectedUpdatedAt?: string | null;
};

export type ProjectSettingsRepositoryLink = {
  id: string;
  repositoryId: string;
  owner: string;
  name: string;
  fullName: string;
  href: string;
  visibility: string;
  linkType: string;
  isDefault: boolean;
  viewerPermission: string | null;
  linkedBy: ProjectWorkspaceUser | null;
  createdAt: string;
  updatedAt: string;
};

export type ProjectSettingsAccessGrant = {
  id: string;
  user: ProjectWorkspaceUser;
  role: string;
  source: string;
  inherited: boolean;
  updatedAt: string;
};

export type ProjectSettingsTeamOption = {
  id: string;
  slug: string;
  name: string;
  href: string;
};

export type ProjectSettingsTeamGrant = {
  id: string;
  team: ProjectSettingsTeamOption;
  role: string;
  memberCount: number;
  updatedAt: string;
};

export type ProjectSettingsStatusUpdate = {
  id: string;
  status: string;
  label: string;
  body: string | null;
  startDate: string | null;
  targetDate: string | null;
  author: ProjectWorkspaceUser | null;
  createdAt: string;
};

export type ProjectSettings = {
  project: ProjectWorkspaceProject;
  general: {
    title: string;
    description: string | null;
    readme: string | null;
    visibility: string;
    defaultRepositoryId: string | null;
    createdBy: ProjectWorkspaceUser | null;
    createdAt: string;
    updatedAt: string;
    readmeRevisionCount: number;
  };
  policy: {
    ownerKind: string;
    organizationId: string | null;
    projectsEnabled: boolean;
    basePermission: string | null;
    visibilityChangesAllowed: boolean;
    visibilityLockedReason: string | null;
  };
  repositories: ProjectSettingsRepositoryLink[];
  accessGrants: ProjectSettingsAccessGrant[];
  teamGrants: ProjectSettingsTeamGrant[];
  eligibleUsers: ProjectWorkspaceUser[];
  eligibleTeams: ProjectSettingsTeamOption[];
  statusUpdates: ProjectSettingsStatusUpdate[];
  template: {
    isTemplate: boolean;
    templateId: string | null;
    title: string | null;
    description: string | null;
    isPublic: boolean;
    createdAt: string | null;
  };
  dangerState: {
    state: string;
    closedAt: string | null;
    closedBy: ProjectWorkspaceUser | null;
    deletedAt: string | null;
    deletedBy: ProjectWorkspaceUser | null;
    deleteConfirmation: string;
  };
  viewerPermissions: {
    authenticated: boolean;
    viewerRole: string | null;
    canEditGeneral: boolean;
    canChangeVisibility: boolean;
    canLinkRepositories: boolean;
    canPublishStatus: boolean;
    canManageTemplate: boolean;
    canManageAccess: boolean;
    canClose: boolean;
    canReopen: boolean;
    canDelete: boolean;
  };
  unavailableReason: string | null;
};

export type ProjectSettingsFetchResult =
  | { ok: true; settings: ProjectSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type ProjectSettingsUpdateRequest = {
  title: string;
  description?: string | null;
  readme?: string | null;
  visibility?: "public" | "private" | string | null;
  defaultRepositoryId?: string | null;
  expectedUpdatedAt?: string | null;
};

export type ProjectRepositoryLinkRequest = {
  expectedUpdatedAt?: string | null;
};

export type ProjectStatusUpdateRequest = {
  status: "on_track" | "at_risk" | "off_track" | "complete" | string;
  body?: string | null;
  startDate?: string | null;
  targetDate?: string | null;
};

export type ProjectTemplateUpdateRequest = {
  isTemplate: boolean;
  title?: string | null;
  description?: string | null;
  isPublic?: boolean | null;
  expectedUpdatedAt?: string | null;
};

export type ProjectAccessRole = "read" | "write" | "admin";

export type ProjectAccessGrantCreateRequest = {
  targetType: "user" | "team";
  targetId: string;
  role: ProjectAccessRole;
  expectedUpdatedAt?: string | null;
};

export type ProjectAccessGrantUpdateRequest = {
  role: ProjectAccessRole;
  expectedUpdatedAt?: string | null;
};

export type ProjectAccessGrantDeleteRequest = {
  expectedUpdatedAt?: string | null;
};

export type ProjectLifecycleRequest = {
  confirmation?: string | null;
  expectedUpdatedAt?: string | null;
};

export type ProjectDeleteResponse = {
  deleted: true;
  projectId: string;
  destinationHref: string;
};

export type ProjectAccessMutation =
  | {
      action: "add-user";
      userId: string;
      role: ProjectAccessRole;
      expectedUpdatedAt?: string | null;
    }
  | {
      action: "add-team";
      teamId: string;
      role: ProjectAccessRole;
      expectedUpdatedAt?: string | null;
    }
  | {
      action: "update-grant";
      grantId: string;
      role: ProjectAccessRole;
      expectedUpdatedAt?: string | null;
    }
  | {
      action: "remove-grant";
      grantId: string;
      expectedUpdatedAt?: string | null;
    };

export type ProjectFieldCreateRequest = {
  name: string;
  fieldType: "single_select" | "iteration" | "date" | "text" | "number";
};

export type ProjectFieldUpdateRequest = {
  name: string;
  expectedUpdatedAt?: string | null;
};

export type ProjectFieldDeleteRequest = {
  expectedUpdatedAt?: string | null;
};

export type ProjectFieldOptionCreateRequest = {
  name: string;
  color?: string | null;
  description?: string | null;
};

export type ProjectIterationSettingsRequest = {
  startDate: string;
  duration: number;
  durationUnit: "days" | "weeks";
  generatedIterations?: number | null;
  expectedUpdatedAt?: string | null;
};

export type ProjectIterationCreateRequest = {
  name?: string | null;
  startDate?: string | null;
  durationDays?: number | null;
};

export type ProjectIterationUpdateRequest = {
  name: string;
  startDate: string;
  durationDays: number;
};

export type ProjectIterationBreakCreateRequest = {
  name?: string | null;
  startDate: string;
  durationDays?: number | null;
};

export type ProjectFieldOptionUpdateRequest = {
  name: string;
  color?: string | null;
  description?: string | null;
};

export type ProjectFieldOptionReorderRequest = {
  optionIds: string[];
};

export type ProjectViewStateRequest = {
  query: string | null;
  sort: string;
  group: string | null;
  slice: string | null;
  hiddenFieldIds: string[];
  expectedUpdatedAt: string;
};

export type ProjectViewLayoutRequest = {
  layout: "table" | "board" | "roadmap";
  columnFieldId?: string | null;
  swimlaneFieldId?: string | null;
  startFieldId?: string | null;
  targetFieldId?: string | null;
  expectedUpdatedAt: string;
};

export type ProjectRoadmapSettingsRequest = {
  startFieldId: string;
  targetFieldId: string;
  markerFieldIds: string[];
  zoom: "month" | "quarter" | "year" | string;
  expectedUpdatedAt: string;
};

export type ProjectItemFieldValueRequest = {
  value: unknown;
  expectedUpdatedAt?: string | null;
};

export type ProjectItemAddRequest = {
  itemType?: "draft_issue" | "issue" | "pull_request" | string | null;
  title?: string | null;
  body?: string | null;
  url?: string | null;
  issueId?: string | null;
  pullRequestId?: string | null;
  positionAfterItemId?: string | null;
};

export type ProjectItemsBulkAddRequest = {
  items: ProjectItemAddRequest[];
};

export type ProjectItemPositionRequest = {
  beforeItemId?: string | null;
  afterItemId?: string | null;
  groupFieldId?: string | null;
  groupValue?: unknown;
  expectedUpdatedAt?: string | null;
};

export type CopyProjectRequest = {
  title: string;
  includeDraftIssues: boolean;
};

export type CopiedProject = {
  id: string;
  number: number;
  title: string;
  href: string;
  workspaceHref: string;
  owner: string;
  copiedViews: number;
  copiedFields: number;
  copiedWorkflows: number;
  copiedDraftItems: number;
};

export type OrganizationSettingsIdentity = {
  id: string;
  slug: string;
  name: string;
  href: string;
  settingsHref: string;
};

export type OrganizationProfileSettingsFields = {
  displayName: string;
  description: string | null;
  websiteUrl: string | null;
  location: string | null;
  publicEmail: string | null;
  contactEmail: string | null;
  billingEmail: string | null;
  companyName: string | null;
  ownershipType: string;
  profileVisibility: string;
  publicMembersVisible: boolean;
};

export type OrganizationSocialAccount = {
  provider: string;
  value: string;
  position: number;
};

export type OrganizationSettingsViewerState = {
  role: string;
  canEditProfile: boolean;
  canRename: boolean;
  canArchive: boolean;
  canDelete: boolean;
};

export type OrganizationAvatarSettings = {
  avatarUrl: string | null;
  storageConfigured: boolean;
  uploadAvailable: boolean;
  unavailableReason: string | null;
};

export type OrganizationProfileSettings = {
  organization: OrganizationSettingsIdentity;
  profile: OrganizationProfileSettingsFields;
  socialAccounts: OrganizationSocialAccount[];
  viewerState: OrganizationSettingsViewerState;
  avatar: OrganizationAvatarSettings;
};

export type OrganizationProfileSettingsFetchResult =
  | { ok: true; settings: OrganizationProfileSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type OrganizationPolicyPermission = "none" | "read" | "write" | "admin";

export type OrganizationAppAccessRequestPolicy =
  | "owners_only"
  | "owners_and_members";

export type OrganizationMemberPrivilegesPolicies = {
  baseRepositoryPermission: OrganizationPolicyPermission | string;
  membersCanCreatePublicRepositories: boolean;
  membersCanCreatePrivateRepositories: boolean;
  membersCanCreateInternalRepositories: boolean;
  membersCanForkPrivateRepositories: boolean;
  repositoryDiscussionsEnabled: boolean;
  projectsBasePermission: OrganizationPolicyPermission | string;
  pagesPublicPublishing: boolean;
  pagesPrivatePublishing: boolean;
  appAccessRequestPolicy: OrganizationAppAccessRequestPolicy | string;
  membersCanChangeRepositoryVisibility: boolean;
  membersCanDeleteRepositories: boolean;
  membersCanTransferRepositories: boolean;
  membersCanDeleteIssues: boolean;
  membersCanCreateTeams: boolean;
};

export type OrganizationPolicyLock = {
  field: keyof OrganizationMemberPrivilegesPolicies | string;
  enforcedBy: string;
  reason: string;
  href: string | null;
};

export type OrganizationPolicyCapabilities = {
  canUpdate: boolean;
  requiresConfirmationFields: Array<
    keyof OrganizationMemberPrivilegesPolicies | string
  >;
  locks: OrganizationPolicyLock[];
};

export type OrganizationMemberPrivilegesSettings = {
  organization: OrganizationSettingsIdentity;
  policies: OrganizationMemberPrivilegesPolicies;
  capabilities: OrganizationPolicyCapabilities;
  viewerState: OrganizationSettingsViewerState;
};

export type OrganizationMemberPrivilegesFetchResult =
  | { ok: true; settings: OrganizationMemberPrivilegesSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type UpdateOrganizationMemberPrivilegesRequest =
  Partial<OrganizationMemberPrivilegesPolicies> & {
    confirmation?: string;
  };

export type UpdateOrganizationProfileSettingsRequest = {
  displayName?: string;
  description?: string | null;
  websiteUrl?: string | null;
  location?: string | null;
  publicEmail?: string | null;
  contactEmail?: string | null;
  billingEmail?: string | null;
  companyName?: string | null;
  socialAccounts?: Array<Pick<OrganizationSocialAccount, "provider" | "value">>;
};

export type RenameOrganizationRequest = {
  name: string;
};

export type UnlinkSignInMethodResponse = {
  removedId: string;
  settings: AccountSecuritySettings;
};

export type PersonalAccessTokenList = {
  tokens: PersonalAccessTokenSummary[];
  sudo: PersonalAccessTokenSudoState;
};

export type PersonalAccessTokenListFetchResult =
  | { ok: true; list: PersonalAccessTokenList }
  | { ok: false; status: number; code: string | null; message: string };

export type PersonalAccessTokenPermissionChoice = {
  key: string;
  label: string;
  levels: string[];
};

export type PersonalAccessTokenPermissionGroup = {
  key: string;
  label: string;
  permissions: PersonalAccessTokenPermissionChoice[];
};

export type PersonalAccessTokenNewContext = {
  sudo: PersonalAccessTokenSudoState;
  resourceOwners: PersonalAccessTokenResourceOwner[];
  repositories: PersonalAccessTokenRepositorySummary[];
  permissionGroups: PersonalAccessTokenPermissionGroup[];
  defaultExpirationDays: number;
  maxExpirationDays: number;
};

export type PersonalAccessTokenNewContextFetchResult =
  | { ok: true; context: PersonalAccessTokenNewContext }
  | { ok: false; status: number; code: string | null; message: string };

export type CreatePersonalAccessTokenRequest = {
  name: string;
  description?: string;
  type?: "fine_grained" | "classic";
  resourceOwnerId: string;
  repositoryAccess: "all" | "selected" | "none";
  repositoryIds: string[];
  expires_in_days?: number | "never";
  permissions: { key: string; level: string }[];
};

export type CreatePersonalAccessTokenResponse = {
  token: PersonalAccessTokenSummary;
  plainTextToken: string;
  createdAt: string;
};

export type RevokePersonalAccessTokenResponse = {
  token: PersonalAccessTokenSummary;
  revokedAt: string;
};

export type SshKeySummary = {
  id: string;
  title: string;
  keyType: string;
  fingerprintSha256: string;
  accessMode: "read_write" | "read_only" | string;
  source: string;
  lastUsedAt: string | null;
  revokedAt: string | null;
  createdAt: string;
};

export type GpgKeySummary = {
  id: string;
  title: string;
  primaryFingerprint: string;
  keyId: string | null;
  emails: string[];
  source: string;
  lastUsedAt: string | null;
  revokedAt: string | null;
  createdAt: string;
};

export type KeySettings = {
  sshKeys: SshKeySummary[];
  gpgKeys: GpgKeySummary[];
  vigilantMode: boolean;
  sudo: PersonalAccessTokenSudoState;
};

export type KeySettingsFetchResult =
  | { ok: true; settings: KeySettings }
  | { ok: false; status: number; code: string | null; message: string };

export type CreateSshKeyRequest = {
  title: string;
  keyType?: string;
  publicKey: string;
  accessMode?: "read_write" | "read_only";
};

export type CreateSshKeyResponse = {
  sshKey: SshKeySummary;
};

export type RevokeSshKeyResponse = {
  sshKey: SshKeySummary;
  revokedAt: string;
};

export type CreateGpgKeyRequest = {
  title: string;
  armoredPublicKey: string;
};

export type CreateGpgKeyResponse = {
  gpgKey: GpgKeySummary;
};

export type RevokeGpgKeyResponse = {
  gpgKey: GpgKeySummary;
  revokedAt: string;
};

export type UpdateVigilantModeRequest = {
  enabled: boolean;
};

export type UpdateVigilantModeResponse = {
  vigilantMode: boolean;
};

export type UserEmailAddress = {
  id: string;
  email: string;
  isPrimary: boolean;
  isPublic: boolean;
  verified: boolean;
};

export type UserSocialAccount = {
  provider: string;
  handleOrUrl: string;
  position: number;
};

export type UserAvatar = {
  id: string;
  url: string;
  contentType: string;
  byteSize: number;
  createdAt: string;
};

export type PersonalProfileSettings = {
  userId: string;
  login: string;
  displayName: string;
  publicEmailId: string | null;
  publicEmail: string | null;
  emails: UserEmailAddress[];
  bio: string;
  pronouns: string;
  websiteUrl: string;
  company: string;
  location: string;
  displayLocalTime: boolean;
  timeZone: string;
  privateProfile: boolean;
  showPrivateContributionCount: boolean;
  achievementsEnabled: boolean;
  preferredLanguage: string;
  socialAccounts: UserSocialAccount[];
  avatar: UserAvatar | null;
  updatedAt: string;
};

export type AppearanceTheme =
  | "light"
  | "dark"
  | "system"
  | "dark_dimmed"
  | "dark_high_contrast";

export type AppearanceFontSize = "small" | "default" | "large";

export type AppearanceSettings = {
  userId: string;
  theme: AppearanceTheme;
  fontSize: AppearanceFontSize;
  updatedAt: string;
};

export type UpdateAppearanceSettingsRequest = {
  theme?: AppearanceTheme;
  fontSize?: AppearanceFontSize;
};

export type UpdatePersonalProfileSettingsRequest = {
  displayName?: string;
  publicEmailId?: string | null;
  bio?: string;
  pronouns?: string;
  websiteUrl?: string;
  company?: string;
  location?: string;
  displayLocalTime?: boolean;
  timeZone?: string;
  privateProfile?: boolean;
  showPrivateContributionCount?: boolean;
  achievementsEnabled?: boolean;
  preferredLanguage?: string;
  socialAccounts?: UserSocialAccount[];
};

export type UpdateAvatarRequest = {
  action: "upload" | "remove";
  fileName?: string;
  contentType?: string;
  byteSize?: number;
  previewUrl?: string;
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

export type OrganizationPeopleAdminTab =
  | "members"
  | "outsideCollaborators"
  | "pendingCollaborators"
  | "invitations"
  | "failedInvitations"
  | "securityManagers";

export type OrganizationPeopleAdminTabParam =
  | "members"
  | "outside_collaborators"
  | "pending_collaborators"
  | "invitations"
  | "failed_invitations"
  | "security_managers";

export type OrganizationPeopleAdmin = {
  organization: OrganizationSettingsIdentity;
  tab: OrganizationPeopleAdminTab;
  filters: OrganizationPeopleAdminFilters;
  counts: OrganizationPeopleAdminCounts;
  rows: ListEnvelope<OrganizationPeopleAdminRow>;
  invitations: ListEnvelope<OrganizationInvitationRow>;
  exports: OrganizationPeopleAdminExport[];
  viewerState: OrganizationPeopleAdminViewerState;
};

export type OrganizationPeopleAdminRow = {
  userId: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
  href: string;
  role: string;
  membershipVisibility: string;
  outsideCollaborator: boolean;
  securityManager: boolean;
  twoFactorEnabled: boolean;
  hasActiveSession: boolean;
  teamCount: number;
  rolesCount: number;
  membershipSource: string;
  joinedAt: string;
  actionState: OrganizationPeopleAdminActionState;
};

export type OrganizationInvitationRow = {
  id: string;
  invitedUserId: string | null;
  invitedLogin: string | null;
  invitedEmail: string;
  role: string;
  teamCount: number;
  status: string;
  emailDeliveryStatus: string;
  emailDeliveryError: string | null;
  invitedByUserId: string;
  expiresAt: string;
  createdAt: string;
  canRetry: boolean;
  canCancel: boolean;
};

export type OrganizationPeopleAdminFilters = {
  tab: OrganizationPeopleAdminTab;
  query: string | null;
  page: number;
  pageSize: number;
};

export type OrganizationPeopleAdminCounts = {
  members: number;
  outsideCollaborators: number;
  pendingCollaborators: number;
  invitations: number;
  failedInvitations: number;
  securityManagers: number;
};

export type OrganizationPeopleAdminExport = {
  format: "json" | "csv" | string;
  href: string;
  available: boolean;
};

export type OrganizationPeopleAdminActionState = {
  canChangeVisibility: boolean;
  canChangeRole: boolean;
  canRemove: boolean;
  finalOwner: boolean;
  reason: string | null;
};

export type OrganizationPeopleAdminViewerState = {
  role: string;
  canAdminPeople: boolean;
  canInvite: boolean;
  canExport: boolean;
};

export type OrganizationPeopleAdminQuery = {
  tab?: OrganizationPeopleAdminTabParam;
  q?: string;
  page?: number;
  pageSize?: number;
};

export type OrganizationTeamsDirectory = {
  organization: OrganizationSettingsIdentity;
  items: OrganizationTeamSummary[];
  total: number;
  page: number;
  pageSize: number;
  filters: OrganizationTeamsFilters;
  counts: OrganizationTeamsCounts;
  parentOptions: OrganizationTeamParentOption[];
  emptyState: OrganizationTeamsEmptyState;
  viewerState: OrganizationTeamsViewerState;
};

export type OrganizationTeamSummary = {
  id: string;
  slug: string;
  name: string;
  description: string | null;
  href: string;
  visibility: "visible" | "secret" | string;
  mentionable: boolean;
  notificationsEnabled: boolean;
  memberCount: number;
  repositoryCount: number;
  childTeamCount: number;
  parent: OrganizationTeamParentOption | null;
  viewerCapabilities: OrganizationTeamCapabilities;
  updatedAt: string;
};

export type OrganizationTeamDetail = {
  organization: OrganizationSettingsIdentity;
  team: OrganizationTeamSummary;
  hierarchy: OrganizationTeamHierarchy;
  members: OrganizationTeamMemberRow[];
  repositories: OrganizationTeamRepositoryPermission[];
  childTeams: OrganizationTeamSummary[];
  mentionState: OrganizationTeamMentionState;
  viewerState: OrganizationTeamsViewerState;
};

export type OrganizationTeamHierarchy = {
  parentChain: OrganizationTeamParentOption[];
  inheritedRepositoryCount: number;
  directRepositoryCount: number;
  childTeamCount: number;
};

export type OrganizationTeamMemberRow = {
  userId: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
  role: string;
  href: string;
};

export type OrganizationTeamRepositoryPermission = {
  repositoryId: string;
  name: string;
  fullName: string;
  href: string;
  visibility: string;
  role: string;
  source: string;
  sourceTeamSlug: string;
  inherited: boolean;
};

export type OrganizationTeamMentionState = {
  mentionable: boolean;
  notificationsEnabled: boolean;
  fanoutState: string;
  recentMentions: OrganizationTeamMentionRow[];
};

export type OrganizationTeamMentionRow = {
  sourceKind: string;
  sourceId: string;
  notificationStatus: string;
  createdAt: string;
};

export type OrganizationTeamParentOption = {
  id: string;
  slug: string;
  name: string;
  href: string;
  visibility: "visible" | "secret" | string;
};

export type OrganizationTeamCapabilities = {
  canView: boolean;
  canManage: boolean;
  canJoin: boolean;
  canMention: boolean;
  isMember: boolean;
};

export type OrganizationTeamsFilters = {
  query: string | null;
  visibility: "all" | "visible" | "secret" | "member" | string;
  page: number;
  pageSize: number;
};

export type OrganizationTeamsCounts = {
  total: number;
  visible: number;
  secret: number;
  memberTeams: number;
};

export type OrganizationTeamsEmptyState = {
  title: string;
  columns: Array<{ title: string; body: string }>;
  newTeamHref: string;
  learnMoreHref: string;
};

export type OrganizationTeamsViewerState = {
  role: string;
  canAdminTeams: boolean;
  canCreateTeam: boolean;
  canViewSecretTeams: boolean;
};

export type OrganizationTeamsQuery = {
  q?: string;
  visibility?: string;
  page?: number;
  pageSize?: number;
};

export type CreateOrganizationTeamRequest = {
  name: string;
  description?: string | null;
  parentTeamId?: string | null;
  visibility: "visible" | "secret";
  notificationsEnabled: boolean;
};

export type OrganizationTeamCreateResult = {
  team: OrganizationTeamSummary;
  destinationHref: string;
};

export type OrganizationInvitationMutation =
  | {
      action: "invite";
      emailOrLogin: string;
      role: "admin" | "member";
      teamIds?: string[];
    }
  | { action: "retry"; invitationId: string }
  | { action: "cancel"; invitationId: string }
  | {
      action: "visibility";
      userId: string;
      visibility: "public" | "private";
    }
  | {
      action: "role";
      userId: string;
      role: "owner" | "admin" | "member";
    }
  | { action: "remove"; userId: string };

export type OwnerPackageList = {
  items: OwnerPackageListItem[];
  total: number;
  page: number;
  pageSize: number;
  owner: OwnerPackageOwner;
  mode: "packages" | string;
  filters: OwnerPackageFilters;
  linkedArtifacts: LinkedArtifactsPlaceholder;
};

export type OwnerPackageOwner = {
  id: string;
  login: string;
  kind: "user" | "organization" | string;
  href: string;
};

export type OwnerPackageListItem = {
  id: string;
  name: string;
  packageType: string;
  typeLabel: string;
  visibility: RepositoryVisibility;
  href: string;
  publishedAt: string;
  publisher: OwnerPackagePublisher;
  linkedRepository: OwnerPackageRepository | null;
  downloadCount: number;
  latestVersion: string | null;
};

export type PackageDetailFetchResult =
  | { ok: true; package: PackageDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type PackageSettingsFetchResult =
  | { ok: true; settings: PackageSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type PackageDetail = {
  id: string;
  name: string;
  packageType: string;
  typeLabel: string;
  visibility: RepositoryVisibility;
  href: string;
  owner: OwnerPackageOwner;
  publisher: OwnerPackagePublisher;
  linkedRepository: OwnerPackageRepository | null;
  publishedAt: string;
  updatedAt: string;
  downloadCount: number;
  selectedVersion: PackageDetailVersion | null;
  versions: PackageDetailVersion[];
  installCommands: PackageInstallCommand[];
  blobs: PackageBlobSummary[];
  about: PackageAboutContent;
  admin: PackageAdminState;
};

export type PackageDetailVersion = {
  id: string;
  version: string;
  digest: string | null;
  shortDigest: string | null;
  platformOs: string | null;
  platformArch: string | null;
  sizeBytes: number | null;
  publishedAt: string;
  publisher: OwnerPackagePublisher;
  href: string;
};

export type PackageBlobSummary = {
  id: string;
  versionId: string | null;
  digest: string;
  shortDigest: string;
  mediaType: string | null;
  platformOs: string | null;
  platformArch: string | null;
  sizeBytes: number | null;
};

export type PackageInstallCommand = {
  label: string;
  command: string;
  version: string | null;
  digest: string | null;
  platform: string | null;
};

export type PackageAboutContent = {
  source: string;
  markdown: string | null;
  html: string | null;
  empty: boolean;
};

export type PackageAdminState = {
  canAdmin: boolean;
  settingsHref: string | null;
  reason: string | null;
};

export type PackageSettings = {
  package: PackageSettingsSummary;
  owner: OwnerPackageOwner;
  linkedRepositories: OwnerPackageRepository[];
  explicitPermissions: PackagePermissionSummary[];
  inheritedRepositoryAccess: PackageRepositoryAccessSummary[];
  recentActivity: PackageActivitySummary[];
  registryWriteCapabilities: PackageCapabilitySummary[];
  admin: PackageAdminState;
};

export type PackageSettingsSummary = {
  id: string;
  name: string;
  packageType: string;
  typeLabel: string;
  visibility: RepositoryVisibility;
  deletedAt: string | null;
  href: string;
  downloadCount: number;
  latestVersionId: string | null;
  latestVersion: string | null;
  latestDigest: string | null;
  updatedAt: string;
};

export type PackagePermissionSummary = {
  userId: string;
  login: string;
  displayName: string | null;
  role: string;
  href: string;
  grantedAt: string;
};

export type PackageRepositoryAccessSummary = {
  repository: OwnerPackageRepository;
  userId: string;
  login: string;
  role: string;
  source: string;
  href: string;
};

export type PackageActivitySummary = {
  kind: string;
  label: string;
  actor: OwnerPackagePublisher | null;
  occurredAt: string;
};

export type PackageCapabilitySummary = {
  key: string;
  label: string;
  enabled: boolean;
  reason: string;
};

export type PackageSettingsMutation =
  | { action: "updateVisibility"; visibility: RepositoryVisibility }
  | {
      action: "grantAccess";
      username: string;
      role: "read" | "write" | "admin";
    }
  | { action: "revokeAccess"; userId: string }
  | { action: "linkRepository"; owner: string; repo: string }
  | { action: "unlinkRepository"; repositoryId: string }
  | { action: "deletePackage" }
  | { action: "restorePackage" }
  | { action: "deleteVersion"; versionId: string }
  | { action: "restoreVersion"; versionId: string };

export type OwnerPackagePublisher = {
  id: string;
  login: string;
  name: string | null;
  href: string;
};

export type OwnerPackageRepository = {
  id: string;
  owner: string;
  name: string;
  fullName: string;
  href: string;
  visibility: RepositoryVisibility;
};

export type OwnerPackageFilters = {
  query: string | null;
  packageType: string;
  visibility: string;
  sort: string;
  artifactTab: string;
  page: number;
  pageSize: number;
};

export type LinkedArtifactsPlaceholder = {
  enabled: boolean;
  message: string;
};

export type OwnerPackageListQuery = {
  q?: string;
  type?: string;
  visibility?: string;
  sort?: string;
  artifactTab?: string;
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

export type ProfileSocialList = {
  items: ProfileSocialListItem[];
  total: number;
  page: number;
  pageSize: number;
  owner: ProfileSocialOwner;
  mode: "followers" | "following" | string;
};

export type ProfileSocialOwner = {
  login: string;
  name: string | null;
  href: string;
};

export type ProfileSocialListItem = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  bio: string | null;
  href: string;
  followedAt: string;
  viewerState: ProfileViewerState;
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

export type RepositoryCommitHistoryView = {
  repository: RepositoryCommitHistoryRepository;
  resolvedRef: RepositoryCommitResolvedRef;
  filters: RepositoryCommitHistoryFilters;
  items: RepositoryCommitListItem[];
  groups: RepositoryCommitGroup[];
  authorOptions: RepositoryCommitAuthorOption[];
  total: number;
  page: number;
  pageSize: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
};

export type RepositoryCommitHistoryRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
};

export type RepositoryCommitResolvedRef = {
  shortName: string;
  qualifiedName: string;
  kind: string;
  targetOid: string | null;
  href: string;
};

export type RepositoryCommitHistoryFilters = {
  path: string | null;
  author: string | null;
  until: string | null;
};

export type RepositoryCommitGroup = {
  date: string;
  commits: RepositoryCommitListItem[];
};

export type RepositoryCommitListItem = {
  oid: string;
  shortOid: string;
  message: string;
  subject: string;
  body: string | null;
  href: string;
  browseHref: string;
  committedAt: string;
  authorLogin: string | null;
  authorAvatarUrl: string | null;
  pullRequests: RepositoryCommitPullRequestLink[];
  status: RepositoryCommitStatusSummary;
  verification: RepositoryCommitVerificationSummary;
};

export type RepositoryCommitPullRequestLink = {
  number: number;
  title: string;
  href: string;
  state: string;
};

export type RepositoryCommitStatusSummary = {
  status: string;
  conclusion: string | null;
  totalCount: number;
  completedCount: number;
  failedCount: number;
  href: string;
};

export type RepositoryCommitVerificationSummary = {
  verified: boolean;
  signatureState: "verified" | "unverified" | "vigilant_unverified" | string;
  signatureSummary: string | null;
};

export type RepositoryCommitAuthorOption = {
  login: string;
  avatarUrl: string | null;
  count: number;
  active: boolean;
};

export type RepositoryCommitDetailView = {
  repository: RepositoryCommitDetailRepository;
  commit: RepositoryCommitDetailCommit;
  parents: RepositoryCommitDetailParent[];
  branches: RepositoryCommitDetailBranchLink[];
  pullRequests: RepositoryCommitPullRequestLink[];
  status: RepositoryCommitStatusSummary;
  verification: RepositoryCommitVerificationSummary;
  diffPlaceholder: RepositoryCommitDetailDiffPlaceholder;
  diffSummary: RepositoryCommitDetailDiffSummary;
  fileTree: RepositoryCommitDetailFileTreeNode[];
  files: RepositoryCommitDetailFile[];
};

export type RepositoryCommitDetailRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  href: string;
  commitHistoryHref: string;
};

export type RepositoryCommitDetailCommit = {
  oid: string;
  shortOid: string;
  message: string;
  subject: string;
  body: string | null;
  href: string;
  browseHref: string;
  committedAt: string;
  authorLogin: string | null;
  authorAvatarUrl: string | null;
  committerLogin: string | null;
  committerAvatarUrl: string | null;
};

export type RepositoryCommitDetailParent = {
  oid: string;
  shortOid: string;
  href: string;
};

export type RepositoryCommitDetailBranchLink = {
  name: string;
  qualifiedName: string;
  kind: string;
  href: string;
};

export type RepositoryCommitDetailDiffPlaceholder = {
  state: string;
  message: string;
  nextPhase: string;
};

export type RepositoryCommitDetailDiffSummary = {
  totalFiles: number;
  additions: number;
  deletions: number;
};

export type RepositoryCommitDetailFileTreeNode = {
  path: string;
  name: string;
  depth: number;
  status: string;
  additions: number;
  deletions: number;
  href: string;
};

export type RepositoryCommitDetailFile = {
  path: string;
  previousPath: string | null;
  status: string;
  additions: number;
  deletions: number;
  byteSize: number;
  blobOid: string | null;
  language: string | null;
  anchor: string;
  href: string;
  rawHref: string;
  viewHref: string;
  isBinary: boolean;
  isLarge: boolean;
  hunks: RepositoryCommitDetailHunk[];
};

export type RepositoryCommitDetailHunk = {
  id: string;
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: RepositoryCommitDetailLine[];
};

export type RepositoryCommitDetailLine = {
  kind: "context" | "added" | "removed" | string;
  oldLine: number | null;
  newLine: number | null;
  content: string;
  position: number;
};

export type RepositoryCommitDetailContext = {
  path: string;
  hunkId: string;
  lines: RepositoryCommitDetailLine[];
  expanded: boolean;
  message: string;
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
  watchLabel?: string;
  watchLevel?: RepositoryWatchLevel;
  customWatchEvents?: RepositoryWatchEvent[];
  forkedRepositoryHref: string | null;
};

export type RepositorySocialState = RepositoryViewerState & {
  starsCount: number;
  watchersCount: number;
  forksCount: number;
};

export type RepositoryStargazerList = {
  items: RepositoryStargazer[];
  total: number;
  page: number;
  pageSize: number;
  repository: {
    ownerLogin: string;
    name: string;
    href: string;
  };
};

export type RepositoryStargazer = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
  bio: string | null;
  href: string;
  starredAt: string;
};

export type RepositoryWatchLevel =
  | "participating"
  | "all"
  | "ignore"
  | "custom";

export type RepositoryWatchEvent =
  | "issues"
  | "pull_requests"
  | "releases"
  | "discussions"
  | "actions"
  | "security_alerts"
  | "repository_invitations";

export type RepositoryWatchSettings = {
  repositoryId: string;
  level: RepositoryWatchLevel;
  label: string;
  watching: boolean;
  watchersCount: number;
  customEvents: RepositoryWatchEvent[];
  availableEvents: RepositoryWatchEvent[];
  ignoreWarning: string;
};

export type RepositoryWatchSettingsPatch = {
  level: RepositoryWatchLevel;
  customEvents?: RepositoryWatchEvent[];
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

export type ReleaseActor = {
  id: string | null;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type ReleaseContributorSummary = {
  id: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type ReleaseReactionSummary = {
  totalCount: number;
  thumbsUp: number;
  thumbsDown: number;
  laugh: number;
  hooray: number;
  confused: number;
  heart: number;
  rocket: number;
  eyes: number;
  viewerReaction: string | null;
};

export type ReleaseReactionContent =
  | "thumbs_up"
  | "thumbs_down"
  | "laugh"
  | "hooray"
  | "confused"
  | "heart"
  | "rocket"
  | "eyes";

export type ReleaseLinks = {
  htmlHref: string;
  apiHref: string;
  tagHref: string;
  zipballHref: string;
  tarballHref: string;
  compareHref: string;
};

export type ReleaseAsset = {
  id: string;
  name: string;
  label: string | null;
  contentType: string;
  byteSize: number;
  downloadCount: number;
  checksumSha256: string | null;
  href: string;
  createdAt: string;
};

export type ReleaseAssetDownloadMetadata = {
  asset: ReleaseAsset;
  releaseId: string;
  releaseTagName: string;
  downloadHref: string;
  authorization: string;
};

export type RepositoryReleaseSummary = {
  id: string;
  tagName: string;
  title: string;
  bodyExcerpt: string | null;
  draft: boolean;
  prerelease: boolean;
  latest: boolean;
  verified: boolean;
  targetOid: string | null;
  shortOid: string | null;
  author: ReleaseActor;
  publishedAt: string | null;
  createdAt: string;
  updatedAt: string;
  assets: ReleaseAsset[];
  reactions: ReleaseReactionSummary;
  contributors: ReleaseContributorSummary[];
  links: ReleaseLinks;
};

export type RepositoryReleaseDetail = RepositoryReleaseSummary & {
  body: string | null;
  bodyHtml: string;
  immutable: boolean;
  tagSignatureSummary: string | null;
};

export type AiOutput = {
  id: string;
  kind: string;
  scopeType: string;
  scopeId: string;
  contentHash: string;
  promptVersion: string;
  model: string;
  output: string;
  generatedAt: string;
  regeneratedCount: number;
  cached: boolean;
};

export type RepositoryAiSummary = {
  enabled: boolean;
  reason: string | null;
  output: AiOutput | null;
};

export type PullRequestAiSummary = {
  enabled: boolean;
  reason: string | null;
  output: AiOutput | null;
  filesOfInterest: Array<{ path: string; note: string }>;
  suggestedReviewers: Array<{ login: string; reason: string }>;
  inlineCommentSeed: string | null;
};

export type AiChangelogRequest = {
  previousTag?: string | null;
  targetTag: string;
};

export type AiChangelog = {
  enabled: boolean;
  reason: string | null;
  output: AiOutput | null;
  previousTag: string | null;
  targetTag: string;
};

export type ReleaseMutation = {
  tagName?: string;
  target?: string;
  title?: string;
  body?: string;
  draft?: boolean;
  prerelease?: boolean;
  latestPolicy?: string;
  deleteTag?: boolean;
};

export type ReleaseAssetMutation = {
  name: string;
  label?: string;
  contentType?: string;
  byteSize?: number;
  checksumSha256?: string;
};

export type ReleaseTagSummary = {
  id: string;
  name: string;
  targetOid: string | null;
  shortOid: string | null;
  commitMessage: string | null;
  committedAt: string | null;
  verified: boolean;
  signatureSummary: string | null;
  releaseId: string | null;
  releaseHref: string | null;
  zipballHref: string;
  tarballHref: string;
  compareHref: string;
};

export type RepositoryReleaseListQuery = {
  page?: number;
  pageSize?: number;
};

export type ReleaseRefOption = {
  name: string;
  shortName: string;
  kind: string;
  targetOid: string | null;
  shortOid: string | null;
  committedAt: string | null;
};

export type ReleaseLatestPolicyOption = {
  value: string;
  label: string;
  description: string;
};

export type ReleaseUploadLimits = {
  maxAssetBytes: number;
  maxAssetCount: number;
  allowedStorageKinds: string[];
  expiresInSeconds: number;
};

export type ReleaseManagementContext = {
  repositoryId: string;
  ownerLogin: string;
  name: string;
  canWrite: boolean;
  archived: boolean;
  release: RepositoryReleaseDetail | null;
  availableTags: ReleaseRefOption[];
  availableRefs: ReleaseRefOption[];
  defaultTarget: string;
  previousTagCandidates: ReleaseRefOption[];
  latestPolicyOptions: ReleaseLatestPolicyOption[];
  uploadLimits: ReleaseUploadLimits;
};

export type GeneratedReleaseNotesPreview = {
  title: string;
  body: string;
  target: ReleaseRefOption;
  previousTag: ReleaseRefOption | null;
  commitCount: number;
  mergedPullRequestCount: number;
  contributors: ReleaseContributorSummary[];
};

export type ReleaseUploadIntent = {
  id: string;
  assetName: string;
  contentType: string;
  byteSize: number;
  checksumSha256: string | null;
  storageKind: string;
  uploadUrl: string;
  handoffToken: string;
  status: string;
  expiresAt: string;
};

export type ReleaseUploadIntentRequest = {
  name: string;
  contentType?: string | null;
  byteSize: number;
  checksumSha256?: string | null;
};

export type ReleaseUploadCompleteRequest = {
  releaseId: string;
  handoffToken: string;
  label?: string | null;
  checksumSha256?: string | null;
};

export type GeneratedReleaseNotesRequest = {
  target: string;
  previousTag?: string | null;
  title?: string | null;
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

export type RepositoryPolicyLock = {
  field: string;
  reason: string;
  settingsHref: string;
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
  policyLocks: RepositoryPolicyLock[];
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

export type RepositoryAccessRole =
  | "read"
  | "triage"
  | "write"
  | "maintain"
  | "admin"
  | "owner";

export type RepositoryAccessRoleDefinition = {
  role: RepositoryAccessRole;
  label: string;
  description: string;
  rank: number;
};

export type RepositoryAccessPerson = {
  userId: string;
  login: string;
  displayName: string | null;
  email: string;
  avatarUrl: string | null;
  role: RepositoryAccessRole;
  source: "owner" | "direct" | "team" | "organization" | "inherited" | string;
  sourceText: string;
  teamSlug: string | null;
  teamName: string | null;
  canEdit: boolean;
  canRemove: boolean;
};

export type RepositoryAccessTeam = {
  teamId: string;
  slug: string;
  name: string;
  role: RepositoryAccessRole;
  source: "team" | "inherited" | string;
  sourceText: string;
  memberCount: number;
  href: string;
  canEdit: boolean;
  canRemove: boolean;
};

export type RepositoryInvitation = {
  id: string;
  invitedUserId: string | null;
  invitedEmail: string;
  invitedLogin: string | null;
  role: RepositoryAccessRole;
  status: string;
  emailDeliveryStatus: string;
  invitedByUserId: string;
  expiresAt: string;
  createdAt: string;
  canCancel: boolean;
};

export type RepositoryInviteUserTarget = {
  userId: string;
  login: string;
  displayName: string | null;
  email: string;
  avatarUrl: string | null;
};

export type RepositoryInviteTeamTarget = {
  teamId: string;
  slug: string;
  name: string;
  memberCount: number;
};

export type RepositoryInviteTargets = {
  users: RepositoryInviteUserTarget[];
  teams: RepositoryInviteTeamTarget[];
};

export type RepositoryAccessSettings = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  viewerPermission: string;
  roles: RepositoryAccessRoleDefinition[];
  people: RepositoryAccessPerson[];
  teams: RepositoryAccessTeam[];
  invitations: RepositoryInvitation[];
  inviteTargets: RepositoryInviteTargets;
  auditEvents: RepositorySettingsAuditEvent[];
};

export type RepositoryAccessSettingsFetchResult =
  | { ok: true; settings: RepositoryAccessSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type BranchPolicyEnforcement = "active" | "evaluate" | "disabled";

export type BranchPolicyRequirements = {
  requiredApprovingReviewCount: number;
  requiresUpToDateBranch: boolean;
  requiredStatusChecks: string[];
  requiresConversationResolution: boolean;
  requiresSignedCommits: boolean;
  requiresLinearHistory: boolean;
  requiresMergeQueue: boolean;
  requiresDeployments: boolean;
  requiredDeploymentEnvironments: string[];
  locked: boolean;
  restrictsPushes: boolean;
  allowsForcePushes: boolean;
  allowsDeletions: boolean;
};

export type BypassActor = {
  actorType: string;
  actorId: string;
  label: string;
};

export type RepositoryDefaultBranchSummary = {
  name: string;
  protected: boolean;
  matchingRuleCount: number;
  matchingRulesetCount: number;
  href: string;
};

export type RepositoryBranchRefSummary = {
  name: string;
  protected: boolean;
  matchingRuleCount: number;
  matchingRulesetCount: number;
  updatedAt: string;
};

export type RepositoryBranchRule = {
  id: string;
  pattern: string;
  description: string | null;
  enforcement: BranchPolicyEnforcement;
  matchingBranches: string[];
  matchingBranchCount: number;
  requirements: BranchPolicyRequirements;
  bypassActors: BypassActor[];
  canEdit: boolean;
  canDelete: boolean;
  createdAt: string;
  updatedAt: string;
};

export type RepositoryRuleset = {
  id: string;
  name: string;
  target: string;
  enforcement: BranchPolicyEnforcement;
  patterns: string[];
  matchingBranches: string[];
  matchingBranchCount: number;
  requirements: BranchPolicyRequirements;
  bypassActors: BypassActor[];
  canEdit: boolean;
  canDelete: boolean;
  createdAt: string;
  updatedAt: string;
};

export type RepositoryBranchSettings = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  defaultBranch: string;
  defaultBranchSummary: RepositoryDefaultBranchSummary;
  viewerPermission: string;
  canEdit: boolean;
  refs: RepositoryBranchRefSummary[];
  rules: RepositoryBranchRule[];
  rulesets: RepositoryRuleset[];
  statusCheckSuggestions: string[];
  auditEvents: RepositorySettingsAuditEvent[];
};

export type RepositoryBranchSettingsFetchResult =
  | { ok: true; settings: RepositoryBranchSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryBranchesView = {
  repository: RepositoryBranchesRepository;
  tabs: RepositoryBranchClassificationCounts;
  filters: RepositoryBranchesFilters;
  defaultBranch: RepositoryBranchDirectoryRow | null;
  branches: RepositoryBranchDirectoryRow[];
  total: number;
  page: number;
  pageSize: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
  emptyState: RepositoryBranchesEmptyState;
};

export type RepositoryBranchesRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  viewerPermission: string;
};

export type RepositoryBranchesFilters = {
  tab: "overview" | "active" | "stale" | "all" | string;
  query: string | null;
  staleCutoffDays: number;
};

export type RepositoryBranchClassificationCounts = {
  overview: number;
  active: number;
  stale: number;
  all: number;
  default: number;
};

export type RepositoryBranchDirectoryRow = {
  name: string;
  qualifiedName: string;
  classification: "default" | "active" | "stale" | string;
  isDefault: boolean;
  href: string;
  commitsHref: string;
  activityHref: string;
  latestCommit: RepositoryBranchLatestCommitSummary | null;
  checks: RepositoryBranchCheckSummary;
  protection: RepositoryBranchProtectionSummary;
  ahead: number;
  behind: number;
  pullRequest: RepositoryBranchPullRequestSummary | null;
  capabilities: RepositoryBranchCapabilities;
  updatedAt: string;
};

export type RepositoryBranchLatestCommitSummary = {
  oid: string;
  shortOid: string;
  subject: string;
  href: string;
  committedAt: string;
  authorLogin: string | null;
  authorAvatarUrl: string | null;
};

export type RepositoryBranchCheckSummary = {
  status: string;
  conclusion: string | null;
  totalCount: number;
  completedCount: number;
  failedCount: number;
  href: string;
};

export type RepositoryBranchProtectionSummary = {
  protected: boolean;
  matchingRuleCount: number;
  matchingRulesetCount: number;
  requiredStatusChecks: string[];
  href: string;
};

export type RepositoryBranchPullRequestSummary = {
  number: number;
  title: string;
  state: string;
  draft: boolean;
  href: string;
};

export type RepositoryBranchCapabilities = {
  canCopy: boolean;
  canViewActivity: boolean;
  canViewRules: boolean;
  canDelete: boolean;
  deleteDisabledReason: string | null;
  canRestore: boolean;
  restoreDisabledReason: string | null;
};

export type RepositoryBranchesEmptyState = {
  title: string;
  message: string;
  resetHref: string;
};

export type RepositoryBranchesFetchResult =
  | { ok: true; branches: RepositoryBranchesView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryBranchActivityView = {
  repository: RepositoryBranchesRepository;
  branch: RepositoryBranchDirectoryRow;
  recentCommits: RepositoryCommitListItem[];
  recentPullRequests: RepositoryBranchPullRequestSummary[];
  protectionEvents: RepositoryBranchProtectionEvent[];
  links: RepositoryBranchActivityLinks;
};

export type RepositoryBranchProtectionEvent = {
  sourceType: string;
  name: string;
  enforcement: BranchPolicyEnforcement;
  href: string;
  requiredStatusChecks: string[];
  updatedAt: string;
};

export type RepositoryBranchActivityLinks = {
  branchesHref: string;
  treeHref: string;
  commitsHref: string;
  compareHref: string;
  rulesHref: string;
};

export type RepositoryBranchActivityFetchResult =
  | { ok: true; activity: RepositoryBranchActivityView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryPulseView = {
  repository: RepositoryPulseRepository;
  period: RepositoryPulsePeriod;
  metrics: RepositoryPulseMetric[];
  summary: RepositoryPulseSummary;
  topCommitters: RepositoryPulseCommitter[];
  releases: RepositoryPulseActivityItem[];
  mergedPullRequests: RepositoryPulseActivityItem[];
  issueActivity: RepositoryPulseActivityItem[];
  snapshot: RepositoryPulseSnapshot;
};

export type RepositoryPulseRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  viewerPermission: string;
  href: string;
};

export type RepositoryPulsePeriod = {
  key: string;
  label: string;
  startedAt: string;
  endedAt: string;
};

export type RepositoryPulseMetric = {
  key: string;
  label: string;
  count: number;
  href: string;
};

export type RepositoryPulseSummary = {
  sentence: string;
  commits: number;
  filesChanged: number;
  additions: number;
  deletions: number;
  authors: number;
  mergedPullRequests: number;
  openPullRequests: number;
  closedIssues: number;
  newIssues: number;
  openIssues: number;
  releases: number;
};

export type RepositoryPulseCommitter = {
  userId: string | null;
  login: string;
  authorStatus?: string;
  isBot?: boolean;
  avatarUrl: string | null;
  commits: number;
  filesChanged: number;
  additions: number;
  deletions: number;
  profileHref: string;
  commitsHref: string;
};

export type RepositoryPulseActivityItem = {
  kind: string;
  number: number | null;
  title: string;
  state: string;
  authorLogin: string | null;
  authorProfileHref?: string | null;
  authorStatus?: string;
  authorAvatarUrl: string | null;
  href: string;
  occurredAt: string;
};

export type RepositoryPulseSnapshot = {
  cacheKey: string;
  computedAt: string;
  expiresAt: string;
  stale: boolean;
};

export type RepositoryPulseFetchResult =
  | { ok: true; pulse: RepositoryPulseView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryContributorsView = {
  repository: RepositoryContributorsRepository;
  period: RepositoryContributorsPeriod;
  threshold: RepositoryContributorsThreshold;
  totals: RepositoryContributorsTotals;
  weeks: RepositoryContributorsWeek[];
  contributors: RepositoryContributorRow[];
  snapshot: RepositoryContributorSnapshot;
};

export type RepositoryContributorsRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  viewerPermission: string;
  href: string;
};

export type RepositoryContributorsPeriod = {
  key: string;
  label: string;
  startedAt: string;
  endedAt: string;
  bucketCount: number;
};

export type RepositoryContributorsThreshold = {
  commitLimit: number;
  commitsConsidered: number;
  lineCountsOmitted: boolean;
  message: string;
};

export type RepositoryContributorsTotals = {
  commits: number;
  authors: number;
  additions: number | null;
  deletions: number | null;
};

export type RepositoryContributorsWeek = {
  weekStart: string;
  weekEnd: string;
  commits: number;
  additions: number | null;
  deletions: number | null;
};

export type RepositoryContributorRow = {
  userId: string | null;
  login: string;
  authorStatus: string;
  isBot: boolean;
  avatarUrl: string | null;
  totalCommits: number;
  totalAdditions: number | null;
  totalDeletions: number | null;
  profileHref: string;
  commitsHref: string;
  weeks: RepositoryContributorWeek[];
};

export type RepositoryContributorWeek = {
  weekStart: string;
  commits: number;
  additions: number | null;
  deletions: number | null;
};

export type RepositoryContributorSnapshot = {
  cacheKey: string;
  computedAt: string;
  expiresAt: string;
  stale: boolean;
};

export type RepositoryContributorsFetchResult =
  | { ok: true; contributors: RepositoryContributorsView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecurityOverviewView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  policy: RepositorySecurityPolicySummary;
  features: RepositorySecurityFeatureCard[];
  advisories: RepositorySecurityAdvisorySummary[];
  links: RepositorySecurityLinks;
};

export type RepositorySecurityRepository = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility | string;
  defaultBranch: string;
  securityHref: string;
  policyHref: string;
  advisoriesHref: string;
};

export type RepositorySecurityViewer = {
  permission: string;
  canRead: boolean;
  canWrite: boolean;
  canEditPolicy: boolean;
  canViewPrivateAlertCounts: boolean;
};

export type RepositorySecurityPolicySummary = {
  exists: boolean;
  path: string | null;
  ref: string | null;
  blobOid: string | null;
  contentSha: string | null;
  html: string | null;
  sourceHref: string | null;
  rawHref: string | null;
  historyHref: string | null;
  editHref: string | null;
  updatedAt: string | null;
  emptyState: string;
};

export type RepositorySecurityFeatureCard = {
  key: string;
  label: string;
  status: string;
  summary: string;
  alertCount: number | null;
  privateCount: number | null;
  href: string;
  configHref: string | null;
  updatedAt: string | null;
};

export type RepositorySecurityAdvisorySummary = {
  id: string;
  identifier: string;
  severity: string;
  status: string;
  title: string;
  summary: string;
  packageName: string | null;
  vulnerableRange: string | null;
  href: string;
  publishedAt: string | null;
  updatedAt: string;
};

export type RepositorySecurityAdvisoriesView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  filters: RepositorySecurityAdvisoryFilters;
  counts: RepositorySecurityAdvisoryCounts;
  advisories: RepositorySecurityAdvisoryRow[];
  links: RepositorySecurityAdvisoryLinks;
};

export type RepositorySecurityAdvisoryFilters = {
  state: string;
  severity: string | null;
  query: string | null;
  sort: string;
  page: number;
  pageSize: number;
  total: number;
  hasNextPage: boolean;
};

export type RepositorySecurityAdvisoryCounts = {
  published: number;
  draft: number | null;
  withdrawn: number | null;
};

export type RepositorySecurityAdvisoryRow = {
  id: string;
  ghsaId: string;
  cveId: string | null;
  severity: string;
  state: string;
  title: string;
  summary: string;
  package: RepositorySecurityAdvisoryPackage | null;
  cvss: CvssSummary | null;
  cwes: CweReference[];
  author: RepositorySecurityAdvisoryActor | null;
  href: string;
  publishedAt: string | null;
  updatedAt: string;
};

export type RepositorySecurityAdvisoryDetail = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityAdvisoryViewer;
  advisory: RepositorySecurityAdvisoryRow;
  markdown: RepositorySecurityAdvisoryMarkdown;
  credits: RepositorySecurityAdvisoryCredit[];
  collaborators: RepositorySecurityAdvisoryCollaborator[];
  timeline: RepositorySecurityAdvisoryTimelineEvent[];
  links: RepositorySecurityAdvisoryLinks;
};

export type RepositorySecurityAdvisoryPackage = {
  ecosystem: string | null;
  name: string | null;
  affectedVersions: string | null;
  patchedVersions: string | null;
};

export type CvssSummary = {
  vector: string | null;
  score: number | null;
  metrics: Record<string, unknown>;
};

export type CweReference = {
  id: string;
  name: string;
  href: string | null;
};

export type RepositorySecurityAdvisoryActor = {
  id: string | null;
  login: string;
  avatarUrl: string | null;
  profileHref: string;
};

export type RepositorySecurityAdvisoryCredit = {
  id: string;
  actor: RepositorySecurityAdvisoryActor;
  creditType: string;
  createdAt: string;
};

export type RepositorySecurityAdvisoryCollaborator = {
  id: string;
  actor: RepositorySecurityAdvisoryActor;
  role: string;
  createdAt: string;
};

export type RepositorySecurityAdvisoryTimelineEvent = {
  id: string;
  eventType: string;
  message: string;
  actor: RepositorySecurityAdvisoryActor | null;
  createdAt: string;
};

export type RepositorySecurityAdvisoryMarkdown = {
  summaryMarkdown: string;
  detailsMarkdown: string;
  detailsHtml: string;
};

export type RepositorySecurityAdvisoryViewer = {
  permission: string;
  canRead: boolean;
  canWrite: boolean;
  canEdit: boolean;
  canPublish: boolean;
  canInviteCollaborators: boolean;
};

export type RepositorySecurityAdvisoryLinks = {
  listHref: string;
  newHref: string | null;
  publishedHref: string;
  draftHref: string | null;
  withdrawnHref: string | null;
};

export type RepositorySecurityAdvisoryMutation = {
  title: string;
  summary: string;
  detailsMarkdown: string;
  cveId: string | null;
  severity: string;
  packageEcosystem: string | null;
  packageName: string | null;
  affectedVersions: string | null;
  patchedVersions: string | null;
  cvssVector: string | null;
  cvssScore: number | null;
  cvssMetrics: Record<string, unknown>;
  cwes: CweReference[];
  credits: { login: string; creditType: string }[];
  collaborators: { login: string; role: string }[];
};

export type RepositorySecurityAdvisoryCreate = {
  title: string;
  summary: string | null;
  detailsMarkdown: string | null;
  cveId: string | null;
  severity: string | null;
  packageEcosystem: string | null;
  packageName: string | null;
  affectedVersions: string | null;
  patchedVersions: string | null;
  cvssVector: string | null;
  cvssScore: number | null;
  cvssMetrics: Record<string, unknown> | null;
  cwes: CweReference[];
  credits: { login: string; creditType: string }[];
  collaborators: { login: string; role: string }[];
};

export type RepositorySecurityAdvisoriesQuery = {
  state?: string | null;
  query?: string | null;
  severity?: string | null;
  sort?: string | null;
  page?: string | number | null;
  pageSize?: string | number | null;
};

export type RepositorySecurityLinks = {
  overviewHref: string;
  policyHref: string;
  advisoriesHref: string;
  dependabotHref: string;
  codeScanningHref: string;
  secretScanningHref: string;
};

export type RepositorySecurityOverviewFetchResult =
  | { ok: true; security: RepositorySecurityOverviewView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecurityPolicyView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  policy: RepositorySecurityPolicyDocument;
  links: RepositorySecurityLinks;
};

export type RepositorySecurityPolicyDocument = {
  exists: boolean;
  path: string | null;
  ref: string | null;
  blobOid: string | null;
  contentSha: string | null;
  markdown: string | null;
  html: string | null;
  outline: RepositorySecurityPolicyHeading[];
  sourceHref: string | null;
  rawHref: string | null;
  historyHref: string | null;
  editHref: string | null;
  latestCommit: RepositorySecurityPolicyCommit | null;
  updatedAt: string | null;
  emptyState: string;
};

export type RepositorySecurityPolicyHeading = {
  id: string;
  level: number;
  text: string;
  href: string;
};

export type RepositorySecurityPolicyCommit = {
  oid: string;
  shortOid: string;
  message: string;
  committedAt: string;
  href: string;
};

export type RepositorySecurityPolicyFetchResult =
  | { ok: true; securityPolicy: RepositorySecurityPolicyView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryWikiView = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  state: RepositoryWikiState;
  page: RepositoryWikiPage | null;
  pages: RepositoryWikiPageSummary[];
  sidebar: RepositoryWikiRenderedBlock | null;
  footer: RepositoryWikiRenderedBlock | null;
  clone: RepositoryWikiCloneInfo;
  links: RepositoryWikiLinks;
};

export type RepositoryWikiRepository = {
  id: string;
  ownerLogin: string;
  name: string;
  visibility: string;
  defaultBranch: string;
  wikiEnabled: boolean;
};

export type RepositoryWikiViewer = {
  permission: string | null;
  canRead: boolean;
  canEditWiki: boolean;
};

export type RepositoryWikiState = {
  kind: "ready" | "empty" | "disabled" | "missing_page";
  message: string;
};

export type RepositoryWikiPage = {
  id: string;
  title: string;
  slug: string;
  path: string;
  href: string;
  revision: RepositoryWikiRevision;
  markdown: string;
  html: string;
  contentSha: string;
  outline: RepositoryWikiHeading[];
  editHref: string | null;
  historyHref: string;
};

export type RepositoryWikiPageSummary = {
  id: string;
  title: string;
  slug: string;
  href: string;
  active: boolean;
  hasOutline: boolean;
  updatedAt: string | null;
};

export type RepositoryWikiRevision = {
  id: string;
  author: RepositoryWikiAuthor | null;
  message: string;
  commitOid: string | null;
  shortOid: string | null;
  createdAt: string;
  href: string;
};

export type RepositoryWikiAuthor = {
  id: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
  href: string;
};

export type RepositoryWikiHeading = {
  id: string;
  level: number;
  text: string;
  href: string;
};

export type RepositoryWikiRenderedBlock = {
  title: string;
  slug: string;
  href: string;
  html: string;
  outline: RepositoryWikiHeading[];
};

export type RepositoryWikiCloneInfo = {
  httpsUrl: string;
};

export type RepositoryWikiLinks = {
  homeHref: string;
  newPageHref: string | null;
};

export type RepositoryWikiFetchResult =
  | { ok: true; wiki: RepositoryWikiView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryWikiPagesIndex = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  pages: RepositoryWikiPageSummary[];
  links: RepositoryWikiLinks;
};

export type RepositoryWikiEditView = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  page: RepositoryWikiEditablePage;
  supportedFormats: RepositoryWikiMarkupFormat[];
};

export type RepositoryWikiEditablePage = {
  id: string;
  title: string;
  slug: string;
  path: string;
  markdown: string;
  latestRevisionId: string;
  editMode: string;
};

export type RepositoryWikiMarkupFormat = {
  mode: string;
  label: string;
  extension: string;
};

export type RepositoryWikiSaveRequest = {
  title: string;
  markdown: string;
  message: string;
  editMode?: string;
  expectedRevisionId?: string | null;
};

export type RepositoryWikiPreviewRequest = {
  markdown: string;
  editMode?: string;
};

export type RepositoryWikiPreviewResult = {
  html: string;
  contentSha: string;
  outline: RepositoryWikiHeading[];
};

export type RepositoryWikiMutationResult = {
  page: RepositoryWikiPage;
  gitCommit: RepositoryWikiGitCommitSummary;
  redirectHref: string;
};

export type RepositoryWikiRevertRequest = {
  pageSlug: string;
  baseRevisionId: string;
  expectedHeadRevisionId: string;
};

export type RepositoryWikiRevertResult = {
  page: RepositoryWikiPage;
  gitCommit: RepositoryWikiGitCommitSummary;
  revertEventId: string;
  restoredRevisionId: string;
  redirectHref: string;
};

export type RepositoryWikiGitCommitSummary = {
  id: string;
  oid: string;
  shortOid: string;
  branch: string;
  message: string;
  storagePath: string;
  createdAt: string;
};

export type RepositoryWikiHistoryView = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  scope: RepositoryWikiHistoryScope;
  revisions: RepositoryWikiHistoryRevision[];
  pagination: RepositoryWikiHistoryPagination;
  links: RepositoryWikiHistoryLinks;
};

export type RepositoryWikiHistoryScope = {
  kind: "all_pages" | "page";
  page: RepositoryWikiPageSummary | null;
};

export type RepositoryWikiHistoryRevision = {
  id: string;
  pageId: string;
  pageTitle: string;
  pageSlug: string;
  pageHref: string;
  author: RepositoryWikiAuthor | null;
  message: string;
  commitOid: string | null;
  shortOid: string | null;
  createdAt: string;
  href: string;
  revisionHref: string;
};

export type RepositoryWikiHistoryPagination = {
  page: number;
  pageSize: number;
  hasNewer: boolean;
  hasOlder: boolean;
  newerHref: string | null;
  olderHref: string | null;
};

export type RepositoryWikiHistoryLinks = {
  homeHref: string;
  pagesHref: string;
  historyHref: string;
};

export type RepositoryWikiHistoryFetchResult =
  | { ok: true; history: RepositoryWikiHistoryView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryWikiRevisionView = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  page: RepositoryWikiPage;
  revisionContext: RepositoryWikiRevisionContext;
  pages: RepositoryWikiPageSummary[];
  links: RepositoryWikiRevisionLinks;
};

export type RepositoryWikiRevisionContext = {
  selectedRevision: RepositoryWikiRevision;
  latestHref: string;
  historyHref: string;
  previousRevisionHref: string | null;
  nextRevisionHref: string | null;
  isLatest: boolean;
};

export type RepositoryWikiRevisionLinks = {
  homeHref: string;
  pagesHref: string;
};

export type RepositoryWikiRevisionFetchResult =
  | { ok: true; revision: RepositoryWikiRevisionView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryWikiCompareView = {
  repository: RepositoryWikiRepository;
  viewer: RepositoryWikiViewer;
  page: RepositoryWikiComparePageSummary;
  base: RepositoryWikiCompareRevisionSummary;
  head: RepositoryWikiCompareRevisionSummary;
  files: RepositoryWikiDiffFile[];
  stats: RepositoryWikiDiffStats;
  links: RepositoryWikiCompareLinks;
};

export type RepositoryWikiComparePageSummary = {
  id: string;
  title: string;
  slug: string;
  href: string;
};

export type RepositoryWikiCompareRevisionSummary = {
  id: string;
  author: RepositoryWikiAuthor | null;
  message: string;
  commitOid: string | null;
  shortOid: string | null;
  createdAt: string;
  href: string;
};

export type RepositoryWikiDiffStats = {
  additions: number;
  deletions: number;
  totalLines: number;
  truncated: boolean;
};

export type RepositoryWikiDiffFile = {
  path: string;
  oldPath: string;
  newPath: string;
  additions: number;
  deletions: number;
  hunks: RepositoryWikiDiffHunk[];
};

export type RepositoryWikiDiffHunk = {
  header: string;
  lines: RepositoryWikiDiffLine[];
};

export type RepositoryWikiDiffLine = {
  kind: "context" | "addition" | "deletion";
  oldNumber: number | null;
  newNumber: number | null;
  content: string;
};

export type RepositoryWikiCompareLinks = {
  historyHref: string;
  baseRevisionHref: string;
  headRevisionHref: string;
  pageHref: string;
};

export type RepositoryWikiCompareFetchResult =
  | { ok: true; compare: RepositoryWikiCompareView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecurityAdvisoriesFetchResult =
  | { ok: true; advisories: RepositorySecurityAdvisoriesView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecurityAdvisoryDetailFetchResult =
  | { ok: true; advisory: RepositorySecurityAdvisoryDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecurityPolicyMutation = {
  markdown: string;
  commitMessage: string;
  path?: string | null;
  ref?: string | null;
  expectedContentSha?: string | null;
};

export type RepositoryDependabotAlertsView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositoryDependabotAvailability;
  filters: RepositoryDependabotAlertFilters;
  counts: RepositoryDependabotAlertCounts;
  alerts: RepositoryDependabotAlertRow[];
  packages: RepositoryDependabotPackageFilter[];
  manifests: RepositoryDependabotManifestFilter[];
  links: RepositoryDependabotLinks;
  freshness: RepositoryDependabotFreshness;
};

export type RepositoryDependabotAlertDetail = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositoryDependabotAvailability;
  alert: RepositoryDependabotAlertRow;
  advisory: RepositoryDependabotAdvisoryDetail;
  dependency: RepositoryDependabotDependencyDetail;
  timeline: RepositoryDependabotTimelineEvent[];
  assigneeOptions: RepositoryDependabotAssignmentOption[];
  securityUpdate: RepositoryDependabotSecurityUpdateState;
  links: RepositoryDependabotLinks;
};

export type RepositoryDependabotAvailability = {
  enabled: boolean;
  indexed: boolean;
  message: string;
  disabledReason: string | null;
  settingsHref: string | null;
};

export type RepositoryDependabotAlertFilters = {
  state: string;
  query: string | null;
  package: string | null;
  ecosystem: string | null;
  manifest: string | null;
  scope: string | null;
  severity: string | null;
  sort: string;
};

export type RepositoryDependabotAlertCounts = {
  open: number;
  closed: number;
  total: number;
  visible: number;
};

export type RepositoryDependabotPackage = {
  id: string;
  ecosystem: string;
  name: string;
  href: string;
};

export type RepositoryDependabotAdvisorySummary = {
  id: string;
  identifier: string;
  severity: string;
  title: string;
  href: string;
  publishedAt: string | null;
};

export type RepositoryDependabotAlertRow = {
  id: string;
  number: number;
  state: string;
  scope: string;
  package: RepositoryDependabotPackage;
  advisory: RepositoryDependabotAdvisorySummary;
  manifestPath: string;
  manifestHref: string;
  lockfilePath: string | null;
  lockfileHref: string | null;
  vulnerableRequirements: string | null;
  currentVersion: string | null;
  fixedVersion: string | null;
  relationship: string;
  assignees: RepositoryDependabotAssignee[];
  href: string;
  detectedAt: string;
  updatedAt: string;
};

export type RepositoryDependabotAssignee = {
  id: string;
  login: string;
  avatarUrl: string | null;
  href: string;
};

export type RepositoryDependabotPackageFilter = {
  package: RepositoryDependabotPackage;
  openCount: number;
  selected: boolean;
};

export type RepositoryDependabotManifestFilter = {
  path: string;
  ecosystem: string;
  href: string;
  openCount: number;
  selected: boolean;
};

export type RepositoryDependabotAdvisoryDetail = {
  identifier: string;
  severity: string;
  title: string;
  href: string;
  vulnerableRange: string;
  publishedAt: string | null;
};

export type RepositoryDependabotDependencyDetail = {
  package: RepositoryDependabotPackage;
  manifestPath: string;
  manifestHref: string;
  lockfilePath: string | null;
  lockfileHref: string | null;
  currentVersion: string | null;
  relationship: string;
};

export type RepositoryDependabotTimelineEvent = {
  id: string;
  eventType: string;
  message: string;
  actor: RepositoryDependabotAssignee | null;
  createdAt: string;
};

export type RepositoryDependabotAssignmentOption = {
  id: string;
  kind: string;
  login: string;
  avatarUrl: string | null;
  selected: boolean;
};

export type RepositoryDependabotSecurityUpdateState = {
  supported: boolean;
  status: string;
  href: string | null;
  pullRequestHref: string | null;
  message: string;
};

export type RepositoryDependabotBulkMutation = {
  action: "dismiss" | "reopen";
  alertIds: string[];
  dismissalReason?: string | null;
  dismissalComment?: string | null;
};

export type RepositoryDependabotBulkMutationResult = {
  repository: RepositorySecurityRepository;
  requestedCount: number;
  updatedCount: number;
  results: RepositoryDependabotBulkAlertResult[];
  message: string;
};

export type RepositoryDependabotBulkAlertResult = {
  id: string;
  number: number;
  state: string;
  ok: boolean;
  message: string;
  href: string;
};

export type RepositoryDependabotSecurityUpdateResult = {
  alert: RepositoryDependabotAlertRow;
  status: string;
  branch: string;
  commitOid: string | null;
  pullRequestHref: string | null;
  message: string;
};

export type RepositoryCodeScanningAlertsView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositoryCodeScanningAvailability;
  filters: RepositoryCodeScanningFilters;
  counts: RepositoryCodeScanningAlertCounts;
  alerts: RepositoryCodeScanningAlertRow[];
  tools: RepositoryCodeScanningToolStatus[];
  branches: RepositoryCodeScanningBranchFilter[];
  links: RepositoryCodeScanningLinks;
  freshness: RepositoryDependabotFreshness;
};

export type RepositoryCodeScanningAlertDetail = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositoryCodeScanningAvailability;
  alert: RepositoryCodeScanningAlertRow;
  location: RepositoryCodeScanningLocation;
  rule: RepositoryCodeScanningRuleDetail;
  timeline: RepositoryCodeScanningTimelineEvent[];
  assigneeOptions: RepositoryDependabotAssignmentOption[];
  linkedIssue: RepositoryCodeScanningLinkedIssueState;
  links: RepositoryCodeScanningLinks;
};

export type RepositoryCodeScanningAvailability = {
  enabled: boolean;
  indexed: boolean;
  message: string;
  disabledReason: string | null;
  settingsHref: string | null;
};

export type RepositoryCodeScanningFilters = {
  state: string;
  query: string | null;
  severity: string | null;
  securitySeverity: string | null;
  tool: string | null;
  branch: string | null;
  ref: string | null;
  tag: string | null;
  applicationCode: string | null;
  sort: string;
};

export type RepositoryCodeScanningAlertCounts = {
  open: number;
  closed: number;
  total: number;
  visible: number;
};

export type RepositoryCodeScanningAlertRow = {
  id: string;
  number: number;
  state: string;
  ruleId: string;
  ruleName: string;
  message: string;
  severity: string;
  securitySeverity: string | null;
  toolName: string;
  path: string;
  pathHref: string;
  startLine: number;
  endLine: number | null;
  refName: string;
  branchName: string | null;
  isDefaultBranch: boolean;
  linkedIssue: RepositoryCodeScanningIssueLink | null;
  assignees: RepositoryDependabotAssignee[];
  href: string;
  detectedAt: string;
  updatedAt: string;
};

export type RepositoryCodeScanningIssueLink = {
  id: string;
  number: number;
  title: string;
  href: string;
};

export type RepositoryCodeScanningLocation = {
  path: string;
  pathHref: string;
  rawHref: string;
  startLine: number;
  endLine: number | null;
  codeSnippet: string | null;
  refName: string;
  commitOid: string | null;
};

export type RepositoryCodeScanningRuleDetail = {
  id: string;
  name: string;
  description: string | null;
  helpMarkdown: string | null;
  helpUri: string | null;
};

export type RepositoryCodeScanningTimelineEvent = {
  id: string;
  eventType: string;
  message: string;
  actor: RepositoryDependabotAssignee | null;
  createdAt: string;
};

export type RepositoryCodeScanningLinkedIssueState = {
  issue: RepositoryCodeScanningIssueLink | null;
  canLink: boolean;
  createHref: string | null;
};

export type RepositoryCodeScanningToolStatus = {
  name: string;
  version: string | null;
  status: string;
  alertCount: number;
  latestRunAt: string | null;
};

export type RepositoryCodeScanningBranchFilter = {
  name: string;
  openCount: number;
  selected: boolean;
};

export type RepositoryCodeScanningLinks = {
  listHref: string;
  openHref: string;
  closedHref: string;
  uploadHref: string;
  settingsHref: string;
};

export type RepositorySecretScanningAvailability = {
  enabled: boolean;
  indexed: boolean;
  pushProtectionEnabled: boolean;
  message: string;
  disabledReason: string | null;
  settingsHref: string | null;
};

export type RepositorySecretScanningFilters = {
  state: string;
  query: string | null;
  provider: string | null;
  secretType: string | null;
  validity: string | null;
  resolution: string | null;
  bypassed: string | null;
  team: string | null;
  topic: string | null;
  sort: string;
};

export type RepositorySecretScanningCounts = {
  open: number;
  resolved: number;
  provider: number;
  generic: number;
  bypassed: number;
  total: number;
  visible: number;
};

export type RepositorySecretScanningPattern = {
  id: string;
  slug: string;
  provider: string;
  secretType: string;
  displayName: string;
  resultKind: string;
  pushProtectionEnabled: boolean;
};

export type RepositorySecretScanningValidity = {
  status: string;
  provider: string;
  checkedAt: string | null;
  message: string;
};

export type RepositorySecretScanningLocation = {
  path: string;
  pathHref: string;
  rawHref: string;
  commitHref: string | null;
  refName: string;
  branchName: string | null;
  startLine: number;
  endLine: number | null;
  redactedSnippet: string | null;
};

export type RepositorySecretScanningAlertRow = {
  id: string;
  number: number;
  state: string;
  resolution: string | null;
  pattern: RepositorySecretScanningPattern;
  redactedSecret: string;
  redactedContext: string | null;
  fingerprint: string;
  validity: RepositorySecretScanningValidity;
  primaryLocation: RepositorySecretScanningLocation | null;
  assignees: RepositoryDependabotAssignee[];
  bypassed: boolean;
  href: string;
  detectedAt: string;
  updatedAt: string;
};

export type RepositorySecretScanningProviderFilter = {
  provider: string;
  openCount: number;
  selected: boolean;
};

export type RepositorySecretScanningSecretTypeFilter = {
  secretType: string;
  displayName: string;
  provider: string;
  resultKind: string;
  openCount: number;
  selected: boolean;
};

export type RepositoryPushProtectionSummary = {
  enabled: boolean;
  protectedPatternCount: number;
  bypassCount: number;
  pendingReviewCount: number;
  settingsHref: string;
};

export type RepositoryPushProtectionBypass = {
  id: string;
  actor: RepositoryDependabotAssignee | null;
  reason: string;
  status: string;
  refName: string;
  commitOid: string | null;
  path: string | null;
  redactedSnippet: string | null;
  createdAt: string;
};

export type RepositorySecretScanningTimelineEvent = {
  id: string;
  eventType: string;
  message: string;
  actor: RepositoryDependabotAssignee | null;
  createdAt: string;
};

export type RepositorySecretScanningLinks = {
  listHref: string;
  providerHref: string;
  genericHref: string;
  openHref: string;
  resolvedHref: string;
  settingsHref: string;
};

export type RepositorySecretScanningAlertsView = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositorySecretScanningAvailability;
  filters: RepositorySecretScanningFilters;
  counts: RepositorySecretScanningCounts;
  alerts: RepositorySecretScanningAlertRow[];
  providers: RepositorySecretScanningProviderFilter[];
  secretTypes: RepositorySecretScanningSecretTypeFilter[];
  pushProtection: RepositoryPushProtectionSummary;
  links: RepositorySecretScanningLinks;
  freshness: RepositoryDependabotFreshness;
};

export type RepositorySecretScanningAlertDetail = {
  repository: RepositorySecurityRepository;
  viewer: RepositorySecurityViewer;
  availability: RepositorySecretScanningAvailability;
  alert: RepositorySecretScanningAlertRow;
  pattern: RepositorySecretScanningPattern;
  locations: RepositorySecretScanningLocation[];
  validity: RepositorySecretScanningValidity;
  bypasses: RepositoryPushProtectionBypass[];
  timeline: RepositorySecretScanningTimelineEvent[];
  assigneeOptions: RepositoryDependabotAssignmentOption[];
  links: RepositorySecretScanningLinks;
};

export type RepositoryCodeScanningAlertsQuery = {
  state?: string | null;
  query?: string | null;
  severity?: string | null;
  securitySeverity?: string | null;
  tool?: string | null;
  branch?: string | null;
  ref?: string | null;
  tag?: string | null;
  applicationCode?: string | null;
  sort?: string | null;
};

export type RepositorySecretScanningAlertsQuery = {
  state?: string | null;
  query?: string | null;
  provider?: string | null;
  secretType?: string | null;
  validity?: string | null;
  resolution?: string | null;
  bypassed?: string | null;
  team?: string | null;
  topic?: string | null;
  sort?: string | null;
};

export type RepositorySecretScanningAlertsFetchResult =
  | { ok: true; secretScanning: RepositorySecretScanningAlertsView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositorySecretScanningAlertDetailFetchResult =
  | { ok: true; secretScanningAlert: RepositorySecretScanningAlertDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryCodeScanningAlertsFetchResult =
  | { ok: true; codeScanning: RepositoryCodeScanningAlertsView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryCodeScanningAlertDetailFetchResult =
  | { ok: true; codeScanningAlert: RepositoryCodeScanningAlertDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryCodeScanningAlertMutation =
  | {
      action: "dismiss";
      dismissalReason: string;
      dismissalComment?: string | null;
    }
  | {
      action: "reopen";
    }
  | {
      action: "assign";
      assigneeIds: string[];
    }
  | {
      action: "link_issue";
      linkedIssueId: string;
    };

export type RepositorySecretScanningAlertMutation =
  | {
      action: "resolve";
      resolution: string;
      resolutionComment?: string | null;
    }
  | {
      action: "reopen";
    }
  | {
      action: "assign";
      assigneeIds: string[];
    }
  | {
      action: "validity";
      validity: string;
    };

export type RepositoryDependabotLinks = {
  listHref: string;
  openHref: string;
  closedHref: string;
  settingsHref: string;
};

export type RepositoryDependabotFreshness = {
  computedAt: string;
  cadence: string;
};

export type RepositoryDependabotAlertsFetchResult =
  | { ok: true; dependabot: RepositoryDependabotAlertsView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryDependabotAlertDetailFetchResult =
  | { ok: true; dependabotAlert: RepositoryDependabotAlertDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryDependabotAlertMutation =
  | {
      action: "dismiss";
      dismissalReason: string;
      dismissalComment?: string | null;
    }
  | {
      action: "reopen";
    }
  | {
      action: "assign";
      assigneeIds: string[];
    };

export type RepositoryDependabotAlertsQuery = {
  state?: string | null;
  query?: string | null;
  package?: string | null;
  ecosystem?: string | null;
  manifest?: string | null;
  scope?: string | null;
  severity?: string | null;
  sort?: string | null;
};

export type RepositoryTrafficView = {
  repository: RepositoryTrafficRepository;
  window: RepositoryTrafficWindow;
  summaries: RepositoryTrafficSummary;
  clones: RepositoryTrafficSeriesPoint[];
  visitors: RepositoryTrafficSeriesPoint[];
  referrers: RepositoryTrafficReferrer[];
  popularContent: RepositoryTrafficContent[];
  snapshot: RepositoryTrafficSnapshot;
};

export type RepositoryTrafficRepository = {
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  viewerPermission: string;
  href: string;
};

export type RepositoryTrafficWindow = {
  key: string;
  label: string;
  startedOn: string;
  endedOn: string;
  timezone: string;
  dayCount: number;
  clonesUpdateCadence: string;
  visitorsUpdateCadence: string;
  referrersUpdateCadence: string;
  popularContentUpdateCadence: string;
  internalTrafficExcluded: boolean;
};

export type RepositoryTrafficSummary = {
  clonesTotal: number;
  clonesUnique: number;
  visitorsTotal: number;
  visitorsUnique: number;
  referrersTotal: number;
  popularContentTotal: number;
  activeDays: number;
  hasTraffic: boolean;
};

export type RepositoryTrafficSeriesPoint = {
  date: string;
  total: number;
  unique: number;
};

export type RepositoryTrafficReferrer = {
  referrer: string;
  href: string;
  totalViews: number;
  uniqueVisitors: number;
};

export type RepositoryTrafficContent = {
  path: string;
  title: string;
  href: string;
  totalViews: number;
  uniqueVisitors: number;
};

export type RepositoryTrafficSnapshot = {
  cacheKey: string;
  computedAt: string;
  expiresAt: string;
  stale: boolean;
};

export type RepositoryTrafficFetchResult =
  | { ok: true; traffic: RepositoryTrafficView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryNetworkView = {
  repository: RepositoryNetworkRepository;
  summary: RepositoryNetworkSummary;
  forks: RepositoryNetworkForkNode[];
  freshness: RepositoryNetworkFreshness;
  links: RepositoryNetworkLinks;
};

export type RepositoryNetworkRepository = {
  id: string;
  ownerLogin: string;
  name: string;
  defaultBranch: string;
  visibility: RepositoryVisibility | string;
  viewerPermission: string;
  href: string;
  treeHref: string;
};

export type RepositoryNetworkSummary = {
  totalReadableForks: number;
  projectedForks: number;
  hiddenPrivateForks: number;
  copy: string;
  updateNote: string;
};

export type RepositoryNetworkForkNode = {
  repositoryId: string;
  ownerLogin: string;
  ownerAvatarUrl: string | null;
  name: string;
  description: string | null;
  visibility: RepositoryVisibility | string;
  defaultBranch: string;
  isArchived: boolean;
  isStarredByActor: boolean;
  starsCount: number;
  forksCount: number;
  openIssuesCount: number;
  openPullRequestsCount: number;
  createdAt: string;
  updatedAt: string;
  pushedAt: string;
  href: string;
  ownerHref: string;
  treeHref: string;
  networkHref: string;
};

export type RepositoryNetworkFreshness = {
  computedAt: string;
  expiresAt: string;
  stale: boolean;
  cadence: string;
};

export type RepositoryNetworkLinks = {
  forksHref: string;
  treeHref: string;
};

export type RepositoryNetworkFetchResult =
  | { ok: true; network: RepositoryNetworkView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryForksView = {
  repository: RepositoryNetworkRepository;
  filters: RepositoryForkFilters;
  defaults: RepositoryForkDefaults;
  total: number;
  hiddenPrivateForks: number;
  forks: RepositoryForkRow[];
  freshness: RepositoryNetworkFreshness;
  links: RepositoryNetworkLinks;
};

export type RepositoryForkFilters = {
  period: RepositoryForkPeriod;
  repositoryType: RepositoryForkType;
  sort: RepositoryForkSort;
};

export type RepositoryForkPeriod = {
  key: string;
  label: string;
  startedAt: string | null;
  endedAt: string;
};

export type RepositoryForkType =
  | "all"
  | "active"
  | "inactive"
  | "archived"
  | "starred"
  | string;

export type RepositoryForkSort =
  | "most_starred"
  | "recently_pushed"
  | "recently_created"
  | "recently_updated"
  | "name"
  | string;

export type RepositoryForkDefaults = {
  saved: boolean;
  matchesCurrent: boolean;
  periodKey: string;
  repositoryType: string;
  sortKey: string;
  savedAt: string | null;
};

export type RepositoryForkRow = RepositoryNetworkForkNode & {
  active: boolean;
  badges: string[];
};

export type RepositoryForksFetchResult =
  | { ok: true; forks: RepositoryForksView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryForksQuery = {
  period?: string | null;
  repositoryType?: string | null;
  sort?: string | null;
};

export type RepositoryDependenciesView = {
  repository: RepositoryNetworkRepository;
  filters: RepositoryDependencyFilters;
  summary: RepositoryDependencySummary;
  manifests: RepositoryDependencyManifest[];
  dependencies: RepositoryDependencyRow[];
  availability: RepositoryDependencyGraphAvailability;
  export: RepositoryDependencyExportState;
  links: RepositoryDependencyLinks;
  freshness: RepositoryNetworkFreshness;
};

export type RepositoryDependentsView = {
  repository: RepositoryNetworkRepository;
  filters: RepositoryDependentsFilters;
  summary: RepositoryDependentsSummary;
  packages: RepositoryDependentPackage[];
  dependents: RepositoryDependentRow[];
  availability: RepositoryDependencyGraphAvailability;
  links: RepositoryDependencyLinks;
  freshness: RepositoryNetworkFreshness;
};

export type RepositoryDependencyFilters = {
  query: string | null;
  ecosystem: string | null;
  relationship: string | null;
};

export type RepositoryDependentsFilters = {
  package: string | null;
  owner: string | null;
};

export type RepositoryDependencySummary = {
  total: number;
  directCount: number;
  transitiveCount: number;
  ecosystemCounts: RepositoryDependencyEcosystemCount[];
  manifestCount: number;
  advisoryCount: number;
};

export type RepositoryDependentsSummary = {
  repositoryCount: number;
  packageCount: number;
  hiddenPrivateCount: number;
  approximate: boolean;
};

export type RepositoryDependencyEcosystemCount = {
  ecosystem: string;
  count: number;
};

export type RepositoryDependencyManifest = {
  id: string;
  path: string;
  ecosystem: string;
  lockfilePath: string | null;
  dependencyCount: number;
  detectedAt: string;
  href: string;
  lockfileHref: string | null;
};

export type RepositoryDependencyPackage = {
  id: string;
  ecosystem: string;
  name: string;
  href: string;
};

export type RepositoryDependentPackage = {
  package: RepositoryDependencyPackage;
  dependentCount: number;
  selected: boolean;
};

export type RepositoryDependentRow = {
  repositoryId: string;
  ownerLogin: string;
  ownerAvatarUrl: string | null;
  name: string;
  description: string | null;
  visibility: string;
  package: RepositoryDependencyPackage;
  manifestPath: string | null;
  detectedAt: string;
  starsCount: number;
  forksCount: number;
  openIssuesCount: number;
  openPullRequestsCount: number;
  href: string;
  ownerHref: string;
  packageHref: string;
};

export type RepositoryDependencyAdvisorySummary = {
  identifier: string;
  severity: string;
  title: string;
  href: string;
};

export type RepositoryDependencyRow = {
  id: string;
  package: RepositoryDependencyPackage;
  version: string | null;
  relationship: string;
  license: string | null;
  manifestPath: string;
  manifestHref: string;
  lockfilePath: string | null;
  lockfileHref: string | null;
  detectedAt: string;
  advisories: RepositoryDependencyAdvisorySummary[];
  detailsHref: string;
  advisoryHref: string | null;
};

export type RepositoryDependencyGraphAvailability = {
  enabled: boolean;
  indexed: boolean;
  supportedEcosystems: string[];
  message: string;
  unavailableReason: string | null;
};

export type RepositoryDependencyExportState = {
  supported: boolean;
  href: string;
  latestStatus: string | null;
};

export type RepositorySbomExport = {
  id: string;
  status: string;
  format: string;
  artifactSha256: string | null;
  artifactByteSize: number;
  downloadHref: string | null;
  pollHref: string;
  expiresAt: string | null;
  createdAt: string;
  completedAt: string | null;
};

export type RepositoryDependencyLinks = {
  dependenciesHref: string;
  dependentsHref: string;
  exportSbomHref: string;
};

export type RepositoryDependenciesFetchResult =
  | { ok: true; dependencies: RepositoryDependenciesView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryDependentsFetchResult =
  | { ok: true; dependents: RepositoryDependentsView }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryDependenciesQuery = {
  query?: string | null;
  ecosystem?: string | null;
  relationship?: string | null;
};

export type RepositoryDependentsQuery = {
  package?: string | null;
  owner?: string | null;
};

export type WebhookContentType = "json" | "form" | string;

export type WebhookEventSelection = "push" | "everything" | "selected" | string;

export type WebhookDeliveryStatus = "queued" | "delivered" | "failed" | string;

export type WebhookEventDefinition = {
  name: string;
  label: string;
  description: string;
};

export type WebhookDeliverySummary = {
  id: string;
  guid: string;
  event: string;
  status: WebhookDeliveryStatus;
  attemptCount: number;
  responseStatus: number | null;
  durationMs: number | null;
  redeliveryOfId: string | null;
  deliveredAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type WebhookDeliveryDetail = {
  summary: WebhookDeliverySummary;
  requestHeaders: unknown;
  requestBodyExcerpt: string | null;
  requestBodyStorageKey: string | null;
  responseHeaders: unknown;
  responseBodyExcerpt: string | null;
  responseBodyStorageKey: string | null;
  terminalError: string | null;
};

export type RepositoryWebhookSummary = {
  id: string;
  payloadUrl: string;
  contentType: WebhookContentType;
  sslVerify: boolean;
  eventSelection: WebhookEventSelection;
  events: string[];
  active: boolean;
  disabledReason: string | null;
  secretConfigured: boolean;
  secretUpdatedAt: string | null;
  latestDelivery: WebhookDeliverySummary | null;
  createdAt: string;
  updatedAt: string;
};

export type RepositoryWebhookSettings = {
  repositoryId: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  viewerPermission: string;
  canEdit: boolean;
  eventDefinitions: WebhookEventDefinition[];
  hooks: RepositoryWebhookSummary[];
};

export type OrganizationWebhookSettings = {
  organizationId: string;
  slug: string;
  name: string;
  viewerRole: string;
  canEdit: boolean;
  eventDefinitions: WebhookEventDefinition[];
  hooks: RepositoryWebhookSummary[];
};

export type RepositoryWebhookDetail = {
  hook: RepositoryWebhookSummary;
  deliveries: WebhookDeliverySummary[];
};

export type RepositoryWebhookSettingsFetchResult =
  | { ok: true; settings: RepositoryWebhookSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type OrganizationWebhookSettingsFetchResult =
  | { ok: true; settings: OrganizationWebhookSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryWebhookDetailFetchResult =
  | { ok: true; detail: RepositoryWebhookDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type WebhookDeliveryDetailFetchResult =
  | { ok: true; delivery: WebhookDeliveryDetail }
  | { ok: false; status: number; code: string | null; message: string };

export type ActionsSettingScope = {
  kind: string;
  name: string | null;
};

export type ActionsSettingActor = {
  id: string;
  login: string;
  displayName: string;
};

export type ActionsSecretSummary = {
  id: string;
  name: string;
  scope: ActionsSettingScope;
  secretConfigured: boolean;
  storageKind: string;
  visibilityPolicy: string;
  updatedBy: ActionsSettingActor | null;
  createdAt: string;
  updatedAt: string;
};

export type ActionsVariableSummary = {
  id: string;
  name: string;
  value: string | null;
  scope: ActionsSettingScope;
  visibilityPolicy: string;
  updatedBy: ActionsSettingActor | null;
  createdAt: string;
  updatedAt: string;
};

export type InheritedActionsSecretSummary = {
  name: string;
  scope: ActionsSettingScope;
  secretConfigured: boolean;
  visibilityPolicy: string;
  updatedAt: string;
};

export type InheritedActionsVariableSummary = {
  name: string;
  value: string | null;
  scope: ActionsSettingScope;
  visibilityPolicy: string;
  updatedAt: string;
};

export type RepositoryActionsSecretsSettings = {
  repositoryId: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  viewerPermission: string;
  canEdit: boolean;
  secrets: ActionsSecretSummary[];
  variables: ActionsVariableSummary[];
  inheritedSecrets: InheritedActionsSecretSummary[];
  inheritedVariables: InheritedActionsVariableSummary[];
};

export type RepositoryActionsSecretsSettingsFetchResult =
  | { ok: true; settings: RepositoryActionsSecretsSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type PagesSourceKind = "none" | "branch" | "actions" | string;

export type PagesSource = {
  kind: PagesSourceKind;
  branch: string | null;
  folder: string | null;
  workflowId: string | null;
  workflowArtifactName: string | null;
};

export type PagesDnsChallenge = {
  name: string;
  value: string;
  recordType: string;
};

export type PagesDomainState = {
  status: string;
  challenge: PagesDnsChallenge | null;
  lastCheckedAt: string | null;
  warning: string | null;
};

export type PagesSiteSummary = {
  id: string;
  source: PagesSource;
  defaultSiteUrl: string;
  customDomain: string | null;
  domain: PagesDomainState;
  httpsEnforced: boolean;
  certificateStatus: string;
  provisioningStatus: string;
  cloudfrontAlias: string | null;
  latestDeploymentId: string | null;
  unpublishedAt: string | null;
  updatedAt: string;
};

export type PagesBranchRef = {
  name: string;
  targetOid: string | null;
  updatedAt: string;
};

export type PagesFolderOption = {
  value: string;
  label: string;
  exists: boolean;
};

export type PagesWorkflowSuggestion = {
  workflowId: string;
  name: string;
  path: string;
  artifactHint: string;
};

export type PagesDeploymentSummary = {
  id: string;
  source: PagesSource;
  status: string;
  conclusion: string | null;
  defaultUrl: string;
  customDomainUrl: string | null;
  workflowRunId: string | null;
  workflowArtifactId: string | null;
  artifactStorageKey: string | null;
  artifactManifest: Record<string, unknown>;
  buildLogExcerpt: string | null;
  failureReason: string | null;
  queuedAt: string;
  completedAt: string | null;
  createdAt: string;
};

export type RepositoryPagesSettings = {
  repositoryId: string;
  ownerLogin: string;
  name: string;
  visibility: RepositoryVisibility;
  viewerPermission: string;
  canEdit: boolean;
  site: PagesSiteSummary;
  availableRefs: PagesBranchRef[];
  folderOptions: PagesFolderOption[];
  workflowSuggestions: PagesWorkflowSuggestion[];
  deployments: PagesDeploymentSummary[];
  auditEvents: RepositorySettingsAuditEvent[];
  policyLock: RepositoryPolicyLock | null;
};

export type RepositoryPagesSettingsFetchResult =
  | { ok: true; settings: RepositoryPagesSettings }
  | { ok: false; status: number; code: string | null; message: string };

export type RepositoryPagesMutation =
  | {
      action: "update-source";
      branch?: string | null;
      folder?: string | null;
      kind: "none" | "branch" | "actions";
      workflowArtifactName?: string | null;
      workflowId?: string | null;
    }
  | { action: "save-domain"; domain: string }
  | { action: "remove-domain" }
  | { action: "recheck-dns" }
  | { action: "update-https"; enforced: boolean }
  | { action: "request-deployment" }
  | { action: "unpublish-pages" };

export type RepositoryActionsSecretMutationPayload = {
  name?: string;
  scopeKind?: "repository" | "environment";
  scopeName?: string | null;
  value: string;
};

export type RepositoryActionsVariableMutationPayload = {
  name?: string;
  scopeKind?: "repository" | "environment";
  scopeName?: string | null;
  value: string;
};

export type RepositoryActionsSecretsMutation =
  | ({ action: "create-secret" } & RepositoryActionsSecretMutationPayload)
  | ({
      action: "update-secret";
      currentName: string;
    } & RepositoryActionsSecretMutationPayload)
  | { action: "delete-secret"; name: string }
  | ({ action: "create-variable" } & RepositoryActionsVariableMutationPayload)
  | ({
      action: "update-variable";
      currentName: string;
    } & RepositoryActionsVariableMutationPayload)
  | { action: "delete-variable"; name: string };

export type RepositoryWebhookMutationPayload = {
  payloadUrl: string;
  contentType?: WebhookContentType;
  secret?: string | null;
  sslVerify?: boolean;
  eventSelection?: WebhookEventSelection;
  events?: string[];
  active?: boolean;
};

export type RepositoryWebhookMutation =
  | ({ action: "create-webhook" } & RepositoryWebhookMutationPayload)
  | ({
      action: "update-webhook";
      hookId: string;
    } & RepositoryWebhookMutationPayload)
  | { action: "delete-webhook"; hookId: string }
  | { action: "ping-webhook"; hookId: string }
  | { action: "redeliver-delivery"; hookId: string; deliveryId: string };

export type RepositoryWebhookMutationResult =
  | RepositoryWebhookSettings
  | { settings: RepositoryWebhookSettings; delivery: WebhookDeliverySummary };

export type BranchPolicyMutationRequirements =
  Partial<BranchPolicyRequirements>;

export type BranchPolicyBypassActorMutation = {
  actorType: string;
  actorId: string;
  label: string;
};

export type RepositoryBranchRuleMutation = BranchPolicyMutationRequirements & {
  action: "create-rule" | "update-rule";
  ruleId?: string;
  pattern: string;
  description?: string | null;
  enforcement?: BranchPolicyEnforcement;
  bypassActors?: BranchPolicyBypassActorMutation[];
};

export type RepositoryBranchRulesetMutation =
  BranchPolicyMutationRequirements & {
    action: "create-ruleset" | "update-ruleset";
    rulesetId?: string;
    name: string;
    enforcement?: BranchPolicyEnforcement;
    patterns: string[];
    bypassActors?: BranchPolicyBypassActorMutation[];
  };

export type RepositoryBranchPolicyMutation =
  | RepositoryBranchRuleMutation
  | RepositoryBranchRulesetMutation
  | { action: "delete-rule"; ruleId: string }
  | { action: "delete-ruleset"; rulesetId: string };

export type WritableRepositoryOwner = {
  ownerType: RepositoryOwnerType;
  id: string;
  login: string;
  displayName: string;
  avatarUrl: string | null;
  visibilityOptions?: RepositoryCreationVisibilityOption[];
};

export type RepositoryCreationVisibilityOption = {
  visibility: RepositoryVisibility;
  enabled: boolean;
  reason: string | null;
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

export type OrganizationSlugAvailability = {
  requestedName: string;
  normalizedSlug: string;
  available: boolean;
  reason: string | null;
  reserved: boolean;
  existingKind: string | null;
};

export type CreateOrganizationRequest = {
  name: string;
  contactEmail: string;
  ownershipType: "personal" | "business";
  companyName?: string | null;
  termsAccepted: boolean;
};

export type CreatedOrganization = {
  id: string;
  slug: string;
  displayName: string;
  contactEmail: string;
  ownershipType: "personal" | "business" | string;
  companyName: string | null;
  termsOfServiceType: string;
  role: string;
  href: string;
  settingsHref: string;
  createdAt: string;
};

export type CreateRepositoryRequest = {
  ownerType: RepositoryOwnerType;
  ownerId: string;
  name: string;
  description?: string | null;
  visibility: RepositoryVisibility;
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
  visibility: RepositoryVisibility;
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

export type DiscussionRepositorySummary = {
  id: string;
  owner: string;
  name: string;
  visibility: RepositoryVisibility | string;
  isArchived: boolean;
  href: string;
  discussionsHref: string;
};

export type DiscussionViewer = {
  authenticated: boolean;
  permission: string | null;
  canRead: boolean;
  canVote: boolean;
  canCreate: boolean;
};

export type DiscussionFilterState = {
  query: string;
  label: string | null;
  state: "open" | "closed" | "all" | string;
  answered: boolean | null;
  locked: boolean | null;
  pinned: boolean | null;
  sort: "latest" | "newest" | "top" | "most_commented" | string;
  category: string | null;
  page: number;
  pageSize: number;
};

export type DiscussionCategorySummary = {
  id: string;
  slug: string;
  name: string;
  emoji: string;
  description: string | null;
  count: number;
  openCount: number;
  href: string;
  active: boolean;
};

export type DiscussionLabelSummary = {
  id: string;
  name: string;
  color: string;
  description: string | null;
  count: number;
};

export type DiscussionAuthorSummary = {
  id: string | null;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type DiscussionRow = {
  id: string;
  number: number;
  title: string;
  state: "open" | "closed" | string;
  answered: boolean;
  locked: boolean;
  pinned: boolean;
  category: DiscussionCategorySummary;
  categoryQualifier?: string;
  labels: DiscussionLabelSummary[];
  author: DiscussionAuthorSummary;
  commentsCount: number;
  votesCount: number;
  viewerVoted: boolean;
  pollSummary?: DiscussionPollSummary | null;
  viewerCanVote?: boolean;
  resultsVisible?: boolean;
  viewerVoteOptionIds?: string[];
  pollUnavailableReasons?: string[];
  href: string;
  createdAt: string;
  updatedAt: string;
  lastActivityAt: string;
};

export type DiscussionPollSummary = {
  id: string;
  question: string;
  allowsMultiple: boolean;
  optionCount: number;
  totalVotes: number;
};

export type PinnedDiscussionCard = {
  discussion: DiscussionRow;
  position: number;
  pinnedAt: string;
};

export type HelpfulContributorSummary = {
  user: DiscussionAuthorSummary;
  commentsCount: number;
  helpfulCount: number;
};

export type CommunityLinkSummary = {
  id: string;
  label: string;
  href: string;
  kind: string;
};

export type RepositoryDiscussionsView = {
  repository: DiscussionRepositorySummary;
  viewer: DiscussionViewer;
  enabled: boolean;
  disabledReason: string | null;
  filters: DiscussionFilterState;
  categories: DiscussionCategorySummary[];
  labels: DiscussionLabelSummary[];
  pinned: PinnedDiscussionCard[];
  helpfulContributors: HelpfulContributorSummary[];
  communityLinks: CommunityLinkSummary[];
  items: DiscussionRow[];
  openCount: number;
  closedCount: number;
  total: number;
  page: number;
  pageSize: number;
  hasNextPage: boolean;
};

export type DiscussionVoteResponse = {
  discussionId: string;
  discussionNumber: number;
  viewerVoted: boolean;
  votesCount: number;
};

export type DiscussionBodyView = {
  markdown: string;
  html: string;
};

export type DiscussionDetailViewer = {
  authenticated: boolean;
  permission: string | null;
  canRead: boolean;
  canComment: boolean;
  canReact: boolean;
  canSubscribe: boolean;
  canMarkAnswer: boolean;
  canModerate: boolean;
  viewerVoted: boolean;
};

export type DiscussionDetailSummary = {
  id: string;
  number: number;
  title: string;
  state: "open" | "closed" | string;
  answered: boolean;
  locked: boolean;
  commentsCount: number;
  votesCount: number;
  href: string;
  createdAt: string;
  updatedAt: string;
  lastActivityAt: string;
};

export type DiscussionFormAnswerView = {
  fieldId: string;
  fieldLabel: string;
  value: string;
};

export type DiscussionPollOptionView = {
  id: string;
  position: number;
  label: string;
  votesCount?: number;
  percentage?: number;
};

export type DiscussionPollView = {
  id: string;
  question: string;
  allowsMultiple: boolean;
  allowsVoteChanges?: boolean;
  totalVotes?: number;
  options: DiscussionPollOptionView[];
  viewerCanVote?: boolean;
  resultsVisible?: boolean;
  viewerVoteOptionIds?: string[];
  unavailableReasons?: string[];
};

export type DiscussionPollVoteRequest = {
  optionIds: string[];
};

export type DiscussionPollVoteResponse = {
  discussionId: string;
  discussionNumber: number;
  poll: DiscussionPollView;
  changed: boolean;
};

export type DiscussionAnswerSummary = {
  commentId: string;
  markedBy: DiscussionAuthorSummary | null;
  markedAt: string;
  href: string;
};

export type DiscussionReactionSummary = {
  content: string;
  count: number;
  viewerReacted: boolean;
};

export type DiscussionSubscriptionState = {
  state: string;
  reason: string | null;
  subscribed: boolean;
  canChange: boolean;
};

export type CreateDiscussionCommentRequest = {
  body: string;
  attachmentDrafts?: DiscussionAttachmentDraft[];
};

export type DiscussionReactionContent =
  | "+1"
  | "-1"
  | "laugh"
  | "hooray"
  | "confused"
  | "heart"
  | "rocket"
  | "eyes";

export type DiscussionEventView = {
  id: string;
  eventType: string;
  actor: DiscussionAuthorSummary | null;
  payload: unknown;
  createdAt: string;
};

export type DiscussionReplyView = {
  id: string;
  author: DiscussionAuthorSummary;
  body: DiscussionBodyView;
  reactions: DiscussionReactionSummary[];
  href: string;
  edited: boolean;
  deleted: boolean;
  deletedReason: string | null;
  createdAt: string;
  updatedAt: string;
};

export type DiscussionCommentView = {
  id: string;
  author: DiscussionAuthorSummary;
  body: DiscussionBodyView;
  reactions: DiscussionReactionSummary[];
  replies: DiscussionReplyView[];
  answer: boolean;
  href: string;
  edited: boolean;
  deleted: boolean;
  deletedReason: string | null;
  createdAt: string;
  updatedAt: string;
};

export type DiscussionTimelineItem =
  | ({ kind: "comment" } & DiscussionCommentView)
  | ({ kind: "event" } & DiscussionEventView);

export type DiscussionSidebarView = {
  category: DiscussionCategorySummary;
  labels: DiscussionLabelSummary[];
  categoryOptions: DiscussionCategoryChoice[];
  labelOptions: DiscussionLabelSummary[];
  participants: DiscussionAuthorSummary[];
  events: DiscussionEventView[];
};

export type DiscussionPinView = {
  target: "global" | "category" | string;
  categorySlug: string | null;
  customTitle: string | null;
  customBody: string | null;
  position: number;
};

export type DiscussionModerationView = {
  globalPin: DiscussionPinView | null;
  categoryPin: DiscussionPinView | null;
  lockAllowsReactions: boolean;
  closedReason: string | null;
};

export type DiscussionTransferTarget = {
  repositoryId: string;
  owner: string;
  name: string;
  visibility: string;
  href: string;
  discussionsHref: string;
  categoryOptions: DiscussionCategoryChoice[];
};

export type DiscussionTransferTargetsView = {
  currentRepository: DiscussionRepositorySummary;
  discussionNumber: number;
  targets: DiscussionTransferTarget[];
};

export type TransferDiscussionResponse = {
  discussionId: string;
  sourceHref: string;
  destinationHref: string;
  destinationOwner: string;
  destinationRepo: string;
  destinationNumber: number;
};

export type DeleteDiscussionResponse = {
  discussionId: string;
  deleted: boolean;
  tombstoneId: string;
  discussionsHref: string;
};

export type RepositoryDiscussionDetailView = {
  repository: DiscussionRepositorySummary;
  viewer: DiscussionDetailViewer;
  enabled: boolean;
  disabledReason: string | null;
  discussion: DiscussionDetailSummary;
  author: DiscussionAuthorSummary;
  category: DiscussionCategorySummary;
  labels: DiscussionLabelSummary[];
  body: DiscussionBodyView;
  formAnswers: DiscussionFormAnswerView[];
  poll: DiscussionPollView | null;
  answer: DiscussionAnswerSummary | null;
  reactions: DiscussionReactionSummary[];
  subscription: DiscussionSubscriptionState;
  moderation: DiscussionModerationView;
  sidebar: DiscussionSidebarView;
  timeline: DiscussionTimelineItem[];
  sort: "oldest" | "newest" | "top" | string;
  page: number;
  pageSize: number;
  totalComments: number;
  hasNextPage: boolean;
};

export type DiscussionCategoryChoice = {
  id: string;
  slug: string;
  name: string;
  emoji: string;
  description: string | null;
  acceptsAnswers: boolean;
  isPoll: boolean;
  count: number;
  openCount: number;
  href: string;
  formHref: string;
};

export type DiscussionCategoryFormat =
  | "announcement"
  | "open_ended"
  | "poll"
  | "question_and_answer"
  | string;

export type DiscussionCategoryAdminViewer = {
  authenticated: boolean;
  permission: string | null;
  canRead: boolean;
  canManage: boolean;
};

export type DiscussionCategorySectionItem = {
  id: string;
  name: string;
  position: number;
  categoryCount: number;
};

export type DiscussionCategoryAdminItem = {
  id: string;
  slug: string;
  name: string;
  emoji: string;
  description: string | null;
  format: DiscussionCategoryFormat;
  acceptsAnswers: boolean;
  isPoll: boolean;
  isDefault: boolean;
  sectionId: string | null;
  sectionName: string | null;
  templatePath: string | null;
  count: number;
  openCount: number;
  position: number;
  href: string;
  editHref: string;
  templateHref: string;
  createdAt: string;
  updatedAt: string;
};

export type DiscussionCategorySettingsView = {
  repository: DiscussionRepositorySummary;
  viewer: DiscussionCategoryAdminViewer;
  enabled: boolean;
  disabledReason: string | null;
  categoryLimit: number;
  remainingCategories: number;
  sections: DiscussionCategorySectionItem[];
  categories: DiscussionCategoryAdminItem[];
};

export type CreateDiscussionCategoryRequest = {
  name: string;
  emoji?: string | null;
  description?: string | null;
  format?: DiscussionCategoryFormat;
  sectionId?: string | null;
};

export type UpdateDiscussionCategoryRequest = {
  name?: string;
  emoji?: string | null;
  description?: string | null;
  format?: DiscussionCategoryFormat;
  sectionId?: string | null;
};

export type CreateDiscussionCategorySectionRequest = {
  name: string;
};

export type UpdateDiscussionCategorySectionRequest = {
  name: string;
};

export type DiscussionCategoryOrderRequest = {
  items: Array<{ id: string; position: number; sectionId: string | null }>;
};

export type DiscussionSectionOrderRequest = {
  items: Array<{ id: string; position: number }>;
};

export type DeleteDiscussionCategoryRequest = {
  moveToCategoryId?: string | null;
};

export type DiscussionCategoryTemplateView = {
  repository: DiscussionRepositorySummary;
  viewer: DiscussionCategoryAdminViewer;
  category: DiscussionCategoryAdminItem;
  path: string;
  content: string;
  contentSha: string;
  branch: string;
  form: DiscussionFormDefinition;
  commitHref: string | null;
  blobHref: string | null;
};

export type DiscussionCategoryTemplatePreviewRequest = {
  content: string;
};

export type DiscussionCategoryTemplateCommitRequest = {
  content: string;
  commitMessage: string;
  branch?: string | null;
  proposeChange?: boolean | null;
  expectedContentSha?: string | null;
};

export type DiscussionCategoryTemplateCommitResponse = {
  template: DiscussionCategoryTemplateView;
  proposed: boolean;
  commitOid: string;
  commitHref: string;
};

export type DiscussionFormField = {
  id: string;
  fieldType: "input" | "textarea" | "dropdown" | "checkboxes" | string;
  label: string;
  description: string | null;
  placeholder: string | null;
  required: boolean;
  options: string[];
};

export type DiscussionFormDefinition = {
  categorySlug: string | null;
  templatePath: string | null;
  title: string;
  description: string | null;
  body: string;
  fields: DiscussionFormField[];
  valid: boolean;
  fallback: boolean;
  parseError: string | null;
};

export type DiscussionSimilarSearch = {
  required: boolean;
  query: string;
  href: string;
};

export type DiscussionCreationView = {
  repository: DiscussionRepositorySummary;
  viewer: DiscussionViewer;
  enabled: boolean;
  disabledReason: string | null;
  categories: DiscussionCategoryChoice[];
  selectedCategory: DiscussionCategoryChoice | null;
  form: DiscussionFormDefinition;
  similarSearch: DiscussionSimilarSearch;
  communityLinks: CommunityLinkSummary[];
};

export type DiscussionFormAnswerInput = {
  fieldId: string;
  value: string;
};

export type DiscussionPollInput = {
  question: string;
  options: string[];
  allowsMultiple?: boolean;
};

export type DiscussionAttachmentDraft = {
  id?: string | null;
  fileName: string;
  contentType: string;
  byteSize: number;
  storageKey: string;
};

export type CreateDiscussionRequest = {
  categorySlug: string;
  title: string;
  body?: string | null;
  similarSearchAcknowledged: boolean;
  formAnswers?: DiscussionFormAnswerInput[];
  poll?: DiscussionPollInput | null;
  attachmentDrafts?: DiscussionAttachmentDraft[];
};

export type CreateDiscussionResponse = {
  discussionId: string;
  discussionNumber: number;
  href: string;
  title: string;
  category: DiscussionCategoryChoice;
};

export type RepositoryDiscussionsQuery = {
  q?: string;
  label?: string;
  state?: string;
  answered?: boolean | string;
  locked?: boolean | string;
  pinned?: boolean | string;
  sort?: string;
  page?: number;
  pageSize?: number;
};

export type RepositoryDiscussionCreationQuery = {
  category?: string;
  title?: string;
};

export type RepositoryDiscussionDetailQuery = {
  sort?: string;
  page?: number;
  pageSize?: number;
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
  retentionDays: number;
  expiredAt: string | null;
  downloadAvailable: boolean;
  deleteAvailable: boolean;
  createdAt: string;
  updatedAt: string;
};

export type ActionsDependencyCache = {
  id: string;
  repositoryId: string;
  key: string;
  version: string;
  scope: string;
  sizeBytes: number;
  lastUsedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type RepositoryActionsCaches = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  viewerPermission: string | null;
  caches: ListEnvelope<ActionsDependencyCache>;
  totalSizeBytes: number;
  limitBytes: number;
  canDelete: boolean;
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

export type ActionsRuntimeScopeCount = {
  scope: string;
  secrets: number;
  variables: number;
};

export type ActionsRuntimePolicy = {
  secretCount: number;
  variableCount: number;
  blockedSecretCount: number;
  blockedVariableCount: number;
  scopes: ActionsRuntimeScopeCount[];
  blockedReasons: string[];
  redactionMarker: string;
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
  runtimePolicy: ActionsRuntimePolicy;
  attempts: ActionsRunAttempt[];
  jobs: ActionsRunJobDetail[];
  annotations: ActionsRunAnnotation[];
  artifacts: ActionsRunArtifact[];
  actionState: ActionsRunActionState;
};

export type ActionsRunnerJob = {
  runId: string;
  jobId: string;
  jobName: string;
  runNumber: number;
  workflowName: string;
  startedAt: string;
};

export type ActionsRunner = {
  id: string;
  name: string;
  labels: string[];
  status: "online" | "offline" | "busy" | string;
  lastHeartbeat: string | null;
  busySince: string | null;
  currentJob: ActionsRunnerJob | null;
  createdAt: string;
  updatedAt: string;
};

export type ActionsRunnerQueue = {
  queuedJobs: number;
  busyRunners: number;
  onlineRunners: number;
  offlineRunners: number;
  concurrencyLimit: number;
  cancelInProgress: boolean;
};

export type ActionsWorkflowPermissions = {
  githubTokenPermission: "read" | "write" | string;
  allowPullRequestApproval: boolean;
  githubTokenScopes: string[];
};

export type ActionsRunnerSetup = {
  registrationToken: string | null;
  dockerCommand: string | null;
  expiresInMinutes: number;
};

export type RepositoryActionsRunnerSettings = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
    defaultBranch: string;
  };
  viewerPermission: string | null;
  canManageRunners: boolean;
  runners: ActionsRunner[];
  queue: ActionsRunnerQueue;
  workflowPermissions: ActionsWorkflowPermissions;
  setup: ActionsRunnerSetup;
};

export type RepositoryActionsRunnerSettingsFetchResult =
  | { ok: true; settings: RepositoryActionsRunnerSettings }
  | {
      ok: false;
      status: number;
      code: string | null;
      message: string;
    };

export type RepositoryActionsRunnerMutation =
  | { action: "create-runner"; name: string; labels: string[] }
  | {
      action: "update-settings";
      concurrencyLimit: number;
      cancelInProgress: boolean;
      githubTokenPermission: string;
      allowPullRequestApproval: boolean;
    }
  | { action: "schedule-jobs" };

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
  customEvents: ThreadSubscriptionEvent[];
  canCustomize: boolean;
};

export type ThreadSubscriptionEvent = "closed" | "reopened" | "merged";

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

export type IssueDiscussionConversionCategory = {
  id: string;
  slug: string;
  name: string;
  emoji: string;
  description: string | null;
  disabledReason: string | null;
};

export type IssueDiscussionConversionView = {
  issueId: string;
  issueNumber: number;
  alreadyConverted: boolean;
  convertedDiscussionNumber: number | null;
  convertedDiscussionHref: string | null;
  categories: IssueDiscussionConversionCategory[];
  commentCount: number;
  canConvert: boolean;
  disabledReason: string | null;
};

export type ConvertIssueToDiscussionResponse = {
  issueId: string;
  issueNumber: number;
  discussionId: string;
  discussionNumber: number;
  href: string;
  title: string;
  categorySlug: string;
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

export type GlobalIssueScope = "created" | "assigned" | "mentioned";

export type GlobalIssueListQuery = {
  scope?: GlobalIssueScope;
  q?: string;
  state?: IssueState;
  repo?: string | null;
  repository?: string | null;
  labels?: string[];
  milestone?: string | null;
  project?: string | null;
  sort?: IssueSort;
  page?: number;
  pageSize?: number;
};

export type GlobalIssueListView = ListEnvelope<IssueListItem> & {
  counts: Record<GlobalIssueScope, number>;
  filters: {
    scope: GlobalIssueScope;
    query: string;
    state: IssueState | null;
    repository: string | null;
    labels: string[];
    milestone: string | null;
    project: string | null;
    sort: IssueSort;
  };
  filterOptions: {
    repositories: {
      id: string;
      ownerLogin: string;
      name: string;
      fullName: string;
      count: number;
    }[];
    labels: IssueListLabel[];
    milestones: IssueListMilestone[];
    projects: IssueListMetadataOption[];
    sortOptions: IssueSort[];
  };
};

export type MilestoneListState = "open" | "closed" | "all";

export type MilestoneSort =
  | "updated-desc"
  | "due-desc"
  | "due-asc"
  | "complete-asc"
  | "complete-desc"
  | "alpha-asc"
  | "alpha-desc"
  | "issues-desc"
  | "issues-asc";

export type RepositoryMilestoneListQuery = {
  state?: MilestoneListState;
  sort?: MilestoneSort;
  page?: number;
  pageSize?: number;
};

export type MilestoneViewer = {
  permission: string | null;
  canEditMilestones: boolean;
};

export type MilestoneOrderState = {
  canReorder: boolean;
  reason: string | null;
  version: string;
};

export type MilestoneProgress = {
  openCount: number;
  closedCount: number;
  totalCount: number;
  percentComplete: number;
};

export type RepositoryMilestoneSummary = {
  id: string;
  title: string;
  description: string | null;
  state: IssueState;
  dueOn: string | null;
  closedAt: string | null;
  createdAt: string;
  updatedAt: string;
  progress: MilestoneProgress;
  openIssuesHref: string;
  closedIssuesHref: string;
  href: string;
};

export type RepositoryMilestonesView =
  ListEnvelope<RepositoryMilestoneSummary> & {
    openCount: number;
    closedCount: number;
    filters: {
      state: MilestoneListState;
      sort: MilestoneSort;
    };
    viewer: MilestoneViewer;
    repository: {
      id: string;
      ownerLogin: string;
      name: string;
      visibility: RepositoryVisibility;
      isArchived: boolean;
    };
  };

export type MilestoneIssueItem = {
  id: string;
  number: number;
  title: string;
  state: IssueState;
  isPullRequest: boolean;
  href: string;
  commentCount: number;
  labelNames: string[];
  assigneeLogins: string[];
  updatedAt: string;
};

export type RepositoryMilestoneDetail = RepositoryMilestoneSummary & {
  descriptionHtml: string;
  items: MilestoneIssueItem[];
  order: MilestoneOrderState;
  viewer: MilestoneViewer;
  repository: RepositoryMilestonesView["repository"];
};

export type RepositoryMilestoneMutation = {
  title: string;
  description?: string | null;
  dueOn?: string | null;
};

export type RepositoryMilestoneOrderRequest = {
  itemIds: string[];
  expectedVersion?: string | null;
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

export type PullRequestChecksView = {
  repository: PullRequestDetailView["repository"];
  pullRequest: {
    id: string;
    number: number;
    title: string;
    state: PullRequestState;
    headRef: string;
    baseRef: string;
    headSha: string | null;
    href: string;
  };
  summary: PullRequestChecksSummary;
  requiredStatusChecks: string[];
  checkRuns: PullRequestCheckRun[];
  canRerun: boolean;
};

export type PullRequestCheckRun = {
  id: string;
  name: string;
  status: string;
  conclusion: string | null;
  required: boolean;
  startedAt: string | null;
  completedAt: string | null;
  outputTitle: string | null;
  outputSummary: string | null;
  annotationsCount: number;
  detailsHref: string | null;
  rerunHref: string | null;
  annotations: PullRequestCheckAnnotation[];
};

export type PullRequestCheckAnnotation = {
  id: string;
  path: string | null;
  startLine: number | null;
  endLine: number | null;
  level: string;
  message: string;
  createdAt: string;
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

export type GlobalPullRequestScope =
  | "created"
  | "assigned"
  | "mentioned"
  | "review_requests";

export type GlobalPullRequestListQuery = {
  scope?: GlobalPullRequestScope | null;
  q?: string | null;
  state?: PullRequestState | null;
  repo?: string | null;
  repository?: string | null;
  labels?: string[] | null;
  milestone?: string | null;
  sort?: PullRequestSort | null;
  page?: number | null;
  pageSize?: number | null;
};

export type GlobalPullRequestListView = ListEnvelope<PullRequestListItem> & {
  counts: Record<GlobalPullRequestScope, number>;
  filters: {
    scope: GlobalPullRequestScope;
    query: string;
    state: PullRequestState | null;
    repository: string | null;
    labels: string[];
    milestone: string | null;
    sort: PullRequestSort;
  };
  filterOptions: {
    repositories: {
      id: string;
      ownerLogin: string;
      name: string;
      fullName: string;
      count: number;
    }[];
    labels: IssueListLabel[];
    milestones: IssueListMilestone[];
    sortOptions: PullRequestSort[];
  };
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
    customEvents: ThreadSubscriptionEvent[];
    canCustomize: boolean;
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
      requiresConversationResolution?: boolean;
      requiresSignedCommits?: boolean;
      requiresLinearHistory?: boolean;
      requiresMergeQueue?: boolean;
      requiresDeployments?: boolean;
      requiredDeploymentEnvironments?: string[];
      locked?: boolean;
      activeRuleCount?: number;
      activeRulesetCount?: number;
      evaluateRuleCount?: number;
      evaluateRulesetCount?: number;
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

export type SearchIndexStatus = {
  documents: {
    kind: string;
    total: number;
    latestIndexedAt: string | null;
  }[];
  events: {
    queued: number;
    running: number;
    completed: number;
    failed: number;
  };
  recentEvents: {
    id: string;
    eventType: string;
    repositoryId: string | null;
    resourceKind: string;
    resourceId: string;
    status: string;
    attempts: number;
    lastError: string | null;
    metadata: Record<string, unknown>;
    completedAt: string | null;
    createdAt: string;
    updatedAt: string;
  }[];
  staleRepositories: {
    repositoryId: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility | string;
    defaultBranch: string;
    latestDocumentIndexedAt: string | null;
    latestEventAt: string | null;
    pendingEvents: number;
    failedEvents: number;
  }[];
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

export async function getApiUserFromCookie(
  cookie: string | null | undefined,
): Promise<ApiUser | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/user`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }
  if (!response.ok) return null;
  return (await response.json()) as ApiUser;
}

function gistListUrl(
  path: string,
  query: { page?: number | null; pageSize?: number | null } = {},
) {
  const url = new URL(`${apiBaseUrl()}${path}`);
  if (query.page && query.page > 1) {
    url.searchParams.set("page", String(query.page));
  }
  if (query.pageSize) {
    url.searchParams.set("pageSize", String(query.pageSize));
  }
  return url;
}

export async function getGistsFromCookie(
  cookie: string | null | undefined,
  query: {
    scope?: "mine" | "public" | string;
    page?: number | null;
    pageSize?: number | null;
  } = {},
): Promise<GistList | null> {
  const url = gistListUrl("/api/gists", query);
  if (query.scope) url.searchParams.set("scope", query.scope);
  try {
    const response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
    if (!response.ok) return null;
    return (await response.json()) as GistList;
  } catch {
    return null;
  }
}

export async function getUserGistsFromCookie(
  cookie: string | null | undefined,
  username: string,
  query: { page?: number | null; pageSize?: number | null } = {},
): Promise<GistList | null> {
  try {
    const response = await fetch(
      gistListUrl(`/api/users/${encodeURIComponent(username)}/gists`, query),
      { headers: cookie ? { cookie } : undefined, cache: "no-store" },
    );
    if (!response.ok) return null;
    return (await response.json()) as GistList;
  } catch {
    return null;
  }
}

export async function getGistFromCookie(
  cookie: string | null | undefined,
  gistId: string,
): Promise<GistDetail | null> {
  try {
    const response = await fetch(
      `${apiBaseUrl()}/api/gists/${encodeURIComponent(gistId)}`,
      { headers: cookie ? { cookie } : undefined, cache: "no-store" },
    );
    if (!response.ok) return null;
    return (await response.json()) as GistDetail;
  } catch {
    return null;
  }
}

export async function getGistRevisionsFromCookie(
  cookie: string | null | undefined,
  gistId: string,
): Promise<GistRevisionList | null> {
  try {
    const response = await fetch(
      `${apiBaseUrl()}/api/gists/${encodeURIComponent(gistId)}/revisions`,
      { headers: cookie ? { cookie } : undefined, cache: "no-store" },
    );
    if (!response.ok) return null;
    return (await response.json()) as GistRevisionList;
  } catch {
    return null;
  }
}

export async function createGistFromCookie(
  cookie: string | null | undefined,
  input: GistMutationRequest,
): Promise<GistDetail> {
  const response = await fetch(`${apiBaseUrl()}/api/gists`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
  });
  if (!response.ok) throw new Error("Gist could not be created");
  return (await response.json()) as GistDetail;
}

export async function updateGistFromCookie(
  cookie: string | null | undefined,
  gistId: string,
  input: GistMutationRequest,
): Promise<GistDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/gists/${encodeURIComponent(gistId)}`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
    },
  );
  if (!response.ok) throw new Error("Gist could not be updated");
  return (await response.json()) as GistDetail;
}

export async function starGistFromCookie(
  cookie: string | null | undefined,
  gistId: string,
  starred: boolean,
): Promise<GistDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/gists/${encodeURIComponent(gistId)}/star`,
    { method: starred ? "PUT" : "DELETE", headers: cookie ? { cookie } : {} },
  );
  if (!response.ok) throw new Error("Gist star could not be changed");
  return (await response.json()) as GistDetail;
}

export async function forkGistFromCookie(
  cookie: string | null | undefined,
  gistId: string,
): Promise<GistDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/gists/${encodeURIComponent(gistId)}/forks`,
    { method: "POST", headers: cookie ? { cookie } : {} },
  );
  if (!response.ok) throw new Error("Gist could not be forked");
  return (await response.json()) as GistDetail;
}

function projectListPath(path: string, query: ProjectListQuery = {}): string {
  const params = new URLSearchParams();
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state?.trim()) {
    params.set("state", query.state.trim());
  }
  if (query.tab?.trim()) {
    params.set("tab", query.tab.trim());
  }
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `${path}${suffix}`;
}

function projectWorkspacePath(
  projectId: string,
  query: ProjectWorkspaceQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.view != null && `${query.view}`.trim() !== "") {
    params.set("view", `${query.view}`);
  }
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.group?.trim()) {
    params.set("group", query.group.trim());
  }
  if (query.slice?.trim()) {
    params.set("slice", query.slice.trim());
  }
  if (query.page != null) {
    params.set("page", String(query.page));
  }
  if (query.pageSize != null) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.toString();
  return `/api/projects/${encodeURIComponent(projectId)}/workspace${suffix ? `?${suffix}` : ""}`;
}

function projectFieldSettingsPath(projectId: string): string {
  return `/api/projects/${encodeURIComponent(projectId)}/settings/fields`;
}

function projectSettingsPath(projectId: string): string {
  return `/api/projects/${encodeURIComponent(projectId)}/settings`;
}

function projectWorkflowSettingsPath(projectId: string): string {
  return `/api/projects/${encodeURIComponent(projectId)}/workflows`;
}

function projectInsightsPath(
  projectId: string,
  query: ProjectInsightsQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.chart?.trim()) {
    params.set("chart", query.chart.trim());
  }
  if (query.range?.trim()) {
    params.set("range", query.range.trim());
  }
  if (query.start?.trim()) {
    params.set("start", query.start.trim());
  }
  if (query.end?.trim()) {
    params.set("end", query.end.trim());
  }
  if (query.filter?.trim()) {
    params.set("filter", query.filter.trim());
  }
  if (query.table != null) {
    params.set("table", String(query.table));
  }
  const suffix = params.toString();
  return `/api/projects/${encodeURIComponent(projectId)}/insights${suffix ? `?${suffix}` : ""}`;
}

function projectItemDetailPath(projectId: string, itemId: string): string {
  return `/api/projects/${encodeURIComponent(projectId)}/items/${encodeURIComponent(itemId)}`;
}

function projectItemDraftPath(projectId: string, itemId: string): string {
  return `${projectItemDetailPath(projectId, itemId)}/draft`;
}

function projectConversionTargetsPath(projectId: string): string {
  return `/api/projects/${encodeURIComponent(projectId)}/conversion-targets`;
}

function projectItemConvertPath(projectId: string, itemId: string): string {
  return `${projectItemDetailPath(projectId, itemId)}/convert-to-issue`;
}

function projectItemCommentsPath(projectId: string, itemId: string): string {
  return `${projectItemDetailPath(projectId, itemId)}/comments`;
}

function projectItemCommentPath(
  projectId: string,
  itemId: string,
  commentId: string,
): string {
  return `${projectItemCommentsPath(projectId, itemId)}/${encodeURIComponent(commentId)}`;
}

function projectArchivedItemsPath(
  projectId: string,
  query: ProjectArchivedItemsQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.itemType && query.itemType !== "all") {
    params.set("itemType", query.itemType);
  }
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.page != null) {
    params.set("page", String(query.page));
  }
  if (query.pageSize != null) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.toString();
  return `/api/projects/${encodeURIComponent(projectId)}/items/archived${suffix ? `?${suffix}` : ""}`;
}

async function getProjectListFromCookie(
  cookie: string | null | undefined,
  path: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${projectListPath(path, query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Projects are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Projects could not be loaded.",
    };
  }

  return { ok: true, projects: (await response.json()) as ProjectList };
}

export function userProjectsPath(
  username: string,
  query: ProjectListQuery = {},
): string {
  return projectListPath(
    `/api/users/${encodeURIComponent(username)}/projects`,
    query,
  );
}

export function organizationProjectsPath(
  org: string,
  query: ProjectListQuery = {},
): string {
  return projectListPath(
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    query,
  );
}

export function repositoryProjectsPath(
  owner: string,
  repo: string,
  query: ProjectListQuery = {},
): string {
  return projectListPath(
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/projects`,
    query,
  );
}

export function getUserProjectsFromCookie(
  cookie: string | null | undefined,
  username: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  return getProjectListFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    query,
  );
}

export function getOrganizationProjectsFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  return getProjectListFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    query,
  );
}

export function getRepositoryProjectsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: ProjectListQuery = {},
): Promise<ProjectListFetchResult> {
  return getProjectListFromCookie(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/projects`,
    query,
  );
}

export async function getProjectWorkspaceFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectWorkspacePath(projectId, query)}`,
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
      message: "Project workspace is unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Project workspace could not be loaded.",
    };
  }

  return {
    ok: true,
    workspace: (await response.json()) as ProjectWorkspace,
  };
}

export async function getProjectItemDetailFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
): Promise<ProjectItemDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectItemDetailPath(projectId, itemId)}`,
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
      message: "Project item detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message:
        body?.error.message ?? "Project item detail could not be loaded.",
    };
  }

  return {
    ok: true,
    detail: (await response.json()) as ProjectItemDetail,
  };
}

export async function getProjectArchivedItemsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  query: ProjectArchivedItemsQuery = {},
): Promise<ProjectArchivedItemsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectArchivedItemsPath(projectId, query)}`,
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
      message: "Archived project items are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message:
        body?.error.message ?? "Archived project items could not be loaded.",
    };
  }

  return {
    ok: true,
    archived: (await response.json()) as ProjectArchivedItems,
  };
}

export async function getProjectFieldSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
): Promise<ProjectFieldSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectFieldSettingsPath(projectId)}`,
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
      message: "Project field settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message:
        body?.error.message ?? "Project field settings could not be loaded.",
    };
  }

  return {
    ok: true,
    settings: (await response.json()) as ProjectFieldSettings,
  };
}

export async function getProjectSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
): Promise<ProjectSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${projectSettingsPath(projectId)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Project settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Project settings could not be loaded.",
    };
  }

  return {
    ok: true,
    settings: (await response.json()) as ProjectSettings,
  };
}

export async function getProjectInsightsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  query: ProjectInsightsQuery = {},
): Promise<ProjectInsightsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectInsightsPath(projectId, query)}`,
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
      message: "Project Insights are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Project Insights could not be loaded.",
    };
  }

  return {
    ok: true,
    insights: (await response.json()) as ProjectInsights,
  };
}

async function mutateProjectInsightsChartFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH" | "DELETE",
  request:
    | ProjectInsightsChartMutationRequest
    | { expectedUpdatedAt?: string | null },
  fallbackMessage: string,
): Promise<ProjectInsights> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(request),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? fallbackMessage, {
      cause: {
        error: envelope?.error ?? {
          code: "project_chart_mutation_failed",
          message: fallbackMessage,
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as ProjectInsights;
}

export function createProjectInsightsChartFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectInsightsChartMutationRequest,
): Promise<ProjectInsights> {
  return mutateProjectInsightsChartFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/charts`,
    "POST",
    request,
    "Project chart could not be created.",
  );
}

export function updateProjectInsightsChartFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  chartId: string,
  request: ProjectInsightsChartMutationRequest,
): Promise<ProjectInsights> {
  return mutateProjectInsightsChartFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/charts/${encodeURIComponent(chartId)}`,
    "PATCH",
    request,
    "Project chart could not be saved.",
  );
}

export function deleteProjectInsightsChartFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  chartId: string,
  request: { expectedUpdatedAt?: string | null },
): Promise<ProjectInsights> {
  return mutateProjectInsightsChartFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/charts/${encodeURIComponent(chartId)}`,
    "DELETE",
    request,
    "Project chart could not be deleted.",
  );
}

async function mutateProjectSettingsFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "PATCH" | "POST" | "DELETE",
  request:
    | ProjectSettingsUpdateRequest
    | ProjectRepositoryLinkRequest
    | ProjectStatusUpdateRequest
    | ProjectTemplateUpdateRequest
    | ProjectAccessGrantCreateRequest
    | ProjectAccessGrantUpdateRequest
    | ProjectAccessGrantDeleteRequest
    | ProjectLifecycleRequest,
  fallbackMessage: string,
): Promise<ProjectSettings> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(request),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? fallbackMessage, {
      cause: {
        error: envelope?.error ?? {
          code: "project_settings_update_failed",
          message: fallbackMessage,
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as ProjectSettings;
}

export function updateProjectSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectSettingsUpdateRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    projectSettingsPath(projectId),
    "PATCH",
    request,
    "Project settings could not be saved.",
  );
}

export function linkProjectRepositoryFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  repositoryId: string,
  request: ProjectRepositoryLinkRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/repositories/${encodeURIComponent(repositoryId)}`,
    "POST",
    request,
    "Project repository could not be linked.",
  );
}

export function unlinkProjectRepositoryFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  repositoryId: string,
  request: ProjectRepositoryLinkRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/repositories/${encodeURIComponent(repositoryId)}`,
    "DELETE",
    request,
    "Project repository could not be removed.",
  );
}

export function createProjectStatusUpdateFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectStatusUpdateRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/status-updates`,
    "POST",
    request,
    "Project status update could not be published.",
  );
}

export function updateProjectTemplateFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectTemplateUpdateRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/template`,
    "PATCH",
    request,
    "Project template settings could not be saved.",
  );
}

export function mutateProjectAccessFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  mutation: ProjectAccessMutation,
): Promise<ProjectSettings> {
  const encodedProjectId = encodeURIComponent(projectId);
  if (mutation.action === "add-user") {
    return mutateProjectSettingsFromCookie(
      cookie,
      `/api/projects/${encodedProjectId}/access-grants`,
      "POST",
      {
        targetType: "user",
        targetId: mutation.userId,
        role: mutation.role,
        expectedUpdatedAt: mutation.expectedUpdatedAt,
      },
      "Project access could not be granted.",
    );
  }
  if (mutation.action === "add-team") {
    return mutateProjectSettingsFromCookie(
      cookie,
      `/api/projects/${encodedProjectId}/access-grants`,
      "POST",
      {
        targetType: "team",
        targetId: mutation.teamId,
        role: mutation.role,
        expectedUpdatedAt: mutation.expectedUpdatedAt,
      },
      "Project team access could not be granted.",
    );
  }
  const encodedGrantId = encodeURIComponent(mutation.grantId);
  if (mutation.action === "update-grant") {
    return mutateProjectSettingsFromCookie(
      cookie,
      `/api/projects/${encodedProjectId}/access-grants/${encodedGrantId}`,
      "PATCH",
      {
        role: mutation.role,
        expectedUpdatedAt: mutation.expectedUpdatedAt,
      },
      "Project access role could not be changed.",
    );
  }
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodedProjectId}/access-grants/${encodedGrantId}`,
    "DELETE",
    { expectedUpdatedAt: mutation.expectedUpdatedAt },
    "Project access grant could not be removed.",
  );
}

export function closeProjectFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectLifecycleRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/close`,
    "POST",
    request,
    "Project could not be closed.",
  );
}

export function reopenProjectFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectLifecycleRequest,
): Promise<ProjectSettings> {
  return mutateProjectSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/reopen`,
    "POST",
    request,
    "Project could not be reopened.",
  );
}

export async function deleteProjectFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectLifecycleRequest,
): Promise<ProjectDeleteResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/projects/${encodeURIComponent(projectId)}`,
    {
      method: "DELETE",
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
      envelope?.error.message ?? "Project could not be deleted.",
      {
        cause: {
          error: envelope?.error ?? {
            code: "project_delete_failed",
            message: "Project could not be deleted.",
          },
          status: envelope?.status ?? response.status,
        } satisfies ApiErrorEnvelope,
      },
    );
  }
  return payload as ProjectDeleteResponse;
}

export async function getProjectWorkflowSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
): Promise<ProjectWorkflowSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${projectWorkflowSettingsPath(projectId)}`,
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
      message: "Project workflows are unavailable right now.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Project workflows could not be loaded.",
    };
  }

  return {
    ok: true,
    settings: (await response.json()) as ProjectWorkflowSettings,
  };
}

export async function updateProjectWorkflowFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  workflowId: string,
  request: ProjectWorkflowUpdateRequest,
): Promise<ProjectWorkflowSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/projects/${encodeURIComponent(projectId)}/workflows/${encodeURIComponent(workflowId)}`,
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
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Project workflow could not be saved.",
      {
        cause: {
          error: envelope?.error ?? {
            code: "project_workflow_update_failed",
            message: "Project workflow could not be saved.",
          },
          status: envelope?.status ?? response.status,
        } satisfies ApiErrorEnvelope,
      },
    );
  }
  return payload as ProjectWorkflowSettings;
}

async function getProjectWorkspaceByNumberFromCookie(
  cookie: string | null | undefined,
  listPath: string,
  projectNumber: number,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  const openProjects = await getProjectListFromCookie(cookie, listPath, {
    state: "open",
    pageSize: 100,
  });
  const closedProjects =
    openProjects.ok &&
    openProjects.projects.items.some(
      (project) => project.number === projectNumber,
    )
      ? null
      : await getProjectListFromCookie(cookie, listPath, {
          state: "closed",
          pageSize: 100,
        });
  const candidates = [
    ...(openProjects.ok ? openProjects.projects.items : []),
    ...(closedProjects?.ok ? closedProjects.projects.items : []),
  ];
  const project = candidates.find((item) => item.number === projectNumber);

  if (!project) {
    const failure = !openProjects.ok ? openProjects : closedProjects;
    return {
      ok: false,
      status: failure && !failure.ok ? failure.status : 404,
      code: failure && !failure.ok ? failure.code : "not_found",
      message:
        failure && !failure.ok
          ? failure.message
          : "Project workspace could not be found.",
    };
  }

  return getProjectWorkspaceFromCookie(cookie, project.id, query);
}

async function getProjectFieldSettingsByNumberFromCookie(
  cookie: string | null | undefined,
  listPath: string,
  projectNumber: number,
): Promise<ProjectFieldSettingsFetchResult> {
  const openProjects = await getProjectListFromCookie(cookie, listPath, {
    state: "open",
    pageSize: 100,
  });
  const closedProjects =
    openProjects.ok &&
    openProjects.projects.items.some(
      (project) => project.number === projectNumber,
    )
      ? null
      : await getProjectListFromCookie(cookie, listPath, {
          state: "closed",
          pageSize: 100,
        });
  const candidates = [
    ...(openProjects.ok ? openProjects.projects.items : []),
    ...(closedProjects?.ok ? closedProjects.projects.items : []),
  ];
  const project = candidates.find((item) => item.number === projectNumber);

  if (!project) {
    const failure = !openProjects.ok ? openProjects : closedProjects;
    return {
      ok: false,
      status: failure && !failure.ok ? failure.status : 404,
      code: failure && !failure.ok ? failure.code : "not_found",
      message:
        failure && !failure.ok
          ? failure.message
          : "Project field settings could not be found.",
    };
  }

  return getProjectFieldSettingsFromCookie(cookie, project.id);
}

async function getProjectSettingsByNumberFromCookie(
  cookie: string | null | undefined,
  listPath: string,
  projectNumber: number,
): Promise<ProjectSettingsFetchResult> {
  const openProjects = await getProjectListFromCookie(cookie, listPath, {
    state: "open",
    pageSize: 100,
  });
  const closedProjects =
    openProjects.ok &&
    openProjects.projects.items.some(
      (project) => project.number === projectNumber,
    )
      ? null
      : await getProjectListFromCookie(cookie, listPath, {
          state: "closed",
          pageSize: 100,
        });
  const candidates = [
    ...(openProjects.ok ? openProjects.projects.items : []),
    ...(closedProjects?.ok ? closedProjects.projects.items : []),
  ];
  const project = candidates.find((item) => item.number === projectNumber);

  if (!project) {
    const failure = !openProjects.ok ? openProjects : closedProjects;
    return {
      ok: false,
      status: failure && !failure.ok ? failure.status : 404,
      code: failure && !failure.ok ? failure.code : "not_found",
      message:
        failure && !failure.ok
          ? failure.message
          : "Project settings could not be found.",
    };
  }

  return getProjectSettingsFromCookie(cookie, project.id);
}

async function getProjectWorkflowSettingsByNumberFromCookie(
  cookie: string | null | undefined,
  listPath: string,
  projectNumber: number,
): Promise<ProjectWorkflowSettingsFetchResult> {
  const openProjects = await getProjectListFromCookie(cookie, listPath, {
    state: "open",
    pageSize: 100,
  });
  const closedProjects =
    openProjects.ok &&
    openProjects.projects.items.some(
      (project) => project.number === projectNumber,
    )
      ? null
      : await getProjectListFromCookie(cookie, listPath, {
          state: "closed",
          pageSize: 100,
        });
  const candidates = [
    ...(openProjects.ok ? openProjects.projects.items : []),
    ...(closedProjects?.ok ? closedProjects.projects.items : []),
  ];
  const project = candidates.find((item) => item.number === projectNumber);

  if (!project) {
    const failure = !openProjects.ok ? openProjects : closedProjects;
    return {
      ok: false,
      status: failure && !failure.ok ? failure.status : 404,
      code: failure && !failure.ok ? failure.code : "not_found",
      message:
        failure && !failure.ok
          ? failure.message
          : "Project workflows could not be found.",
    };
  }

  return getProjectWorkflowSettingsFromCookie(cookie, project.id);
}

async function getProjectInsightsByNumberFromCookie(
  cookie: string | null | undefined,
  listPath: string,
  projectNumber: number,
  query: ProjectInsightsQuery = {},
): Promise<ProjectInsightsFetchResult> {
  const openProjects = await getProjectListFromCookie(cookie, listPath, {
    state: "open",
    pageSize: 100,
  });
  const closedProjects =
    openProjects.ok &&
    openProjects.projects.items.some(
      (project) => project.number === projectNumber,
    )
      ? null
      : await getProjectListFromCookie(cookie, listPath, {
          state: "closed",
          pageSize: 100,
        });
  const candidates = [
    ...(openProjects.ok ? openProjects.projects.items : []),
    ...(closedProjects?.ok ? closedProjects.projects.items : []),
  ];
  const project = candidates.find((item) => item.number === projectNumber);

  if (!project) {
    const failure = !openProjects.ok ? openProjects : closedProjects;
    return {
      ok: false,
      status: failure && !failure.ok ? failure.status : 404,
      code: failure && !failure.ok ? failure.code : "not_found",
      message:
        failure && !failure.ok
          ? failure.message
          : "Project Insights could not be found.",
    };
  }

  return getProjectInsightsFromCookie(cookie, project.id, query);
}

export function getUserProjectWorkspaceFromCookie(
  cookie: string | null | undefined,
  username: string,
  projectNumber: number,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  return getProjectWorkspaceByNumberFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    projectNumber,
    query,
  );
}

export function getUserProjectFieldSettingsFromCookie(
  cookie: string | null | undefined,
  username: string,
  projectNumber: number,
): Promise<ProjectFieldSettingsFetchResult> {
  return getProjectFieldSettingsByNumberFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    projectNumber,
  );
}

export function getUserProjectSettingsFromCookie(
  cookie: string | null | undefined,
  username: string,
  projectNumber: number,
): Promise<ProjectSettingsFetchResult> {
  return getProjectSettingsByNumberFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    projectNumber,
  );
}

export function getUserProjectWorkflowSettingsFromCookie(
  cookie: string | null | undefined,
  username: string,
  projectNumber: number,
): Promise<ProjectWorkflowSettingsFetchResult> {
  return getProjectWorkflowSettingsByNumberFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    projectNumber,
  );
}

export function getUserProjectInsightsFromCookie(
  cookie: string | null | undefined,
  username: string,
  projectNumber: number,
  query: ProjectInsightsQuery = {},
): Promise<ProjectInsightsFetchResult> {
  return getProjectInsightsByNumberFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/projects`,
    projectNumber,
    query,
  );
}

export function getOrganizationProjectWorkspaceFromCookie(
  cookie: string | null | undefined,
  org: string,
  projectNumber: number,
  query: ProjectWorkspaceQuery = {},
): Promise<ProjectWorkspaceFetchResult> {
  return getProjectWorkspaceByNumberFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    projectNumber,
    query,
  );
}

export function getOrganizationProjectFieldSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  projectNumber: number,
): Promise<ProjectFieldSettingsFetchResult> {
  return getProjectFieldSettingsByNumberFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    projectNumber,
  );
}

export function getOrganizationProjectSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  projectNumber: number,
): Promise<ProjectSettingsFetchResult> {
  return getProjectSettingsByNumberFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    projectNumber,
  );
}

export function getOrganizationProjectWorkflowSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  projectNumber: number,
): Promise<ProjectWorkflowSettingsFetchResult> {
  return getProjectWorkflowSettingsByNumberFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    projectNumber,
  );
}

export function getOrganizationProjectInsightsFromCookie(
  cookie: string | null | undefined,
  org: string,
  projectNumber: number,
  query: ProjectInsightsQuery = {},
): Promise<ProjectInsightsFetchResult> {
  return getProjectInsightsByNumberFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/projects`,
    projectNumber,
    query,
  );
}

export async function copyProjectFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: CopyProjectRequest,
): Promise<CopiedProject> {
  const response = await fetch(
    `${apiBaseUrl()}/api/projects/${encodeURIComponent(projectId)}/copies`,
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
    throw new Error(envelope?.error.message ?? "Project could not be copied.", {
      cause: {
        error: envelope?.error ?? {
          code: "project_copy_failed",
          message: "Project could not be copied.",
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as CopiedProject;
}

export async function updateProjectViewStateFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  viewId: string,
  request: ProjectViewStateRequest,
): Promise<ProjectWorkspace> {
  const response = await fetch(
    `${apiBaseUrl()}/api/projects/${encodeURIComponent(projectId)}/views/${encodeURIComponent(viewId)}/state`,
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
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Project view state could not be saved.",
      {
        cause: {
          error: envelope?.error ?? {
            code: "project_view_state_failed",
            message: "Project view state could not be saved.",
          },
          status: envelope?.status ?? response.status,
        } satisfies ApiErrorEnvelope,
      },
    );
  }
  return payload as ProjectWorkspace;
}

async function mutateProjectFieldSettingsFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH" | "DELETE",
  fallbackCode: string,
  fallbackMessage: string,
  body?: unknown,
): Promise<ProjectFieldSettings> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      ...(body == null ? {} : { "content-type": "application/json" }),
      ...(cookie ? { cookie } : {}),
    },
    body: body == null ? undefined : JSON.stringify(body),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? fallbackMessage, {
      cause: {
        error: envelope?.error ?? {
          code: fallbackCode,
          message: fallbackMessage,
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as ProjectFieldSettings;
}

export function createProjectFieldFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectFieldCreateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/settings/fields`,
    "POST",
    "project_field_create_failed",
    "Project field could not be created.",
    request,
  );
}

export function updateProjectFieldFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectFieldUpdateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}`,
    "PATCH",
    "project_field_update_failed",
    "Project field could not be saved.",
    request,
  );
}

export function deleteProjectFieldFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectFieldDeleteRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}`,
    "DELETE",
    "project_field_delete_failed",
    "Project field could not be deleted.",
    request,
  );
}

export function createProjectFieldOptionFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectFieldOptionCreateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/options`,
    "POST",
    "project_field_option_create_failed",
    "Project field option could not be created.",
    request,
  );
}

export function updateProjectFieldOptionFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  optionId: string,
  request: ProjectFieldOptionUpdateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/options/${encodeURIComponent(optionId)}`,
    "PATCH",
    "project_field_option_update_failed",
    "Project field option could not be saved.",
    request,
  );
}

export function reorderProjectFieldOptionsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectFieldOptionReorderRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/options/reorder`,
    "PATCH",
    "project_field_option_reorder_failed",
    "Project field options could not be reordered.",
    request,
  );
}

export function deleteProjectFieldOptionFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  optionId: string,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/options/${encodeURIComponent(optionId)}`,
    "DELETE",
    "project_field_option_delete_failed",
    "Project field option could not be deleted.",
  );
}

export function updateProjectIterationSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectIterationSettingsRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/iterations/settings`,
    "PATCH",
    "project_iteration_settings_failed",
    "Project iteration settings could not be saved.",
    request,
  );
}

export function createProjectIterationFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectIterationCreateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/iterations`,
    "POST",
    "project_iteration_create_failed",
    "Project iteration could not be created.",
    request,
  );
}

export function updateProjectIterationFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  iterationId: string,
  request: ProjectIterationUpdateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/iterations/${encodeURIComponent(iterationId)}`,
    "PATCH",
    "project_iteration_update_failed",
    "Project iteration could not be saved.",
    request,
  );
}

export function createProjectIterationBreakFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  request: ProjectIterationBreakCreateRequest,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/iteration-breaks`,
    "POST",
    "project_iteration_break_create_failed",
    "Project iteration break could not be created.",
    request,
  );
}

export function deleteProjectIterationBreakFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  fieldId: string,
  breakId: string,
): Promise<ProjectFieldSettings> {
  return mutateProjectFieldSettingsFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/fields/${encodeURIComponent(fieldId)}/iteration-breaks/${encodeURIComponent(breakId)}`,
    "DELETE",
    "project_iteration_break_delete_failed",
    "Project iteration break could not be deleted.",
  );
}

export async function updateProjectViewLayoutFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  viewId: string,
  request: ProjectViewLayoutRequest,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/views/${encodeURIComponent(viewId)}/layout`,
    "PATCH",
    "project_view_layout_failed",
    "Project view layout could not be saved.",
    request,
  );
}

export function updateProjectRoadmapSettingsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  viewId: string,
  request: ProjectRoadmapSettingsRequest,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/views/${encodeURIComponent(viewId)}/roadmap-settings`,
    "PATCH",
    "project_roadmap_settings_failed",
    "Project roadmap settings could not be saved.",
    request,
  );
}

export async function updateProjectItemFieldFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  fieldId: string,
  request: ProjectItemFieldValueRequest,
): Promise<ProjectWorkspace> {
  const response = await fetch(
    `${apiBaseUrl()}/api/projects/${encodeURIComponent(projectId)}/items/${encodeURIComponent(itemId)}/fields/${encodeURIComponent(fieldId)}`,
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
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Project item field could not be saved.",
      {
        cause: {
          error: envelope?.error ?? {
            code: "project_item_field_failed",
            message: "Project item field could not be saved.",
          },
          status: envelope?.status ?? response.status,
        } satisfies ApiErrorEnvelope,
      },
    );
  }
  return payload as ProjectWorkspace;
}

async function mutateProjectWorkspaceFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH" | "DELETE",
  fallbackCode: string,
  fallbackMessage: string,
  body?: unknown,
): Promise<ProjectWorkspace> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      ...(body == null ? {} : { "content-type": "application/json" }),
      ...(cookie ? { cookie } : {}),
    },
    body: body == null ? undefined : JSON.stringify(body),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? fallbackMessage, {
      cause: {
        error: envelope?.error ?? {
          code: fallbackCode,
          message: fallbackMessage,
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as ProjectWorkspace;
}

async function mutateProjectItemDetailFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH" | "DELETE",
  fallbackCode: string,
  fallbackMessage: string,
  body?: unknown,
): Promise<ProjectItemDetail> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      ...(body == null ? {} : { "content-type": "application/json" }),
      ...(cookie ? { cookie } : {}),
    },
    body: body == null ? undefined : JSON.stringify(body),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? fallbackMessage, {
      cause: {
        error: envelope?.error ?? {
          code: fallbackCode,
          message: fallbackMessage,
        },
        status: envelope?.status ?? response.status,
      } satisfies ApiErrorEnvelope,
    });
  }
  return payload as ProjectItemDetail;
}

export function addProjectItemFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectItemAddRequest,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/items`,
    "POST",
    "project_item_add_failed",
    "Project item could not be added.",
    request,
  );
}

export function bulkAddProjectItemsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  request: ProjectItemsBulkAddRequest,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/items/bulk`,
    "POST",
    "project_item_bulk_add_failed",
    "Project items could not be added.",
    request,
  );
}

export function updateProjectItemPositionFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  request: ProjectItemPositionRequest,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/items/${encodeURIComponent(itemId)}/position`,
    "PATCH",
    "project_item_position_failed",
    "Project item position could not be saved.",
    request,
  );
}

export function removeProjectItemFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
): Promise<ProjectWorkspace> {
  return mutateProjectWorkspaceFromCookie(
    cookie,
    `/api/projects/${encodeURIComponent(projectId)}/items/${encodeURIComponent(itemId)}`,
    "DELETE",
    "project_item_remove_failed",
    "Project item could not be removed.",
  );
}

export function archiveProjectItemFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    `${projectItemDetailPath(projectId, itemId)}/archive`,
    "PATCH",
    "project_item_archive_failed",
    "Project item could not be archived.",
  );
}

export function restoreProjectItemFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    `${projectItemDetailPath(projectId, itemId)}/restore`,
    "PATCH",
    "project_item_restore_failed",
    "Project item could not be restored.",
  );
}

export function updateProjectDraftItemFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  request: ProjectDraftUpdateRequest,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    projectItemDraftPath(projectId, itemId),
    "PATCH",
    "project_draft_update_failed",
    "Draft project item could not be saved.",
    request,
  );
}

export async function getProjectConversionTargetsFromCookie(
  cookie: string | null | undefined,
  projectId: string,
): Promise<ProjectConversionTargets> {
  const response = await fetch(
    `${apiBaseUrl()}${projectConversionTargetsPath(projectId)}`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Project conversion targets could not be loaded.",
      {
        cause: {
          error: envelope?.error ?? {
            code: "project_conversion_targets_failed",
            message: "Project conversion targets could not be loaded.",
          },
          status: envelope?.status ?? response.status,
        } satisfies ApiErrorEnvelope,
      },
    );
  }
  return payload as ProjectConversionTargets;
}

export function convertProjectDraftToIssueFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  request: ProjectDraftConvertRequest,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    projectItemConvertPath(projectId, itemId),
    "POST",
    "project_draft_convert_failed",
    "Draft project item could not be converted.",
    request,
  );
}

export function createProjectItemCommentFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  request: ProjectItemCommentCreateRequest,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    projectItemCommentsPath(projectId, itemId),
    "POST",
    "project_item_comment_failed",
    "Project item comment could not be saved.",
    request,
  );
}

export function updateProjectItemCommentFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  commentId: string,
  request: ProjectItemCommentUpdateRequest,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    projectItemCommentPath(projectId, itemId, commentId),
    "PATCH",
    "project_item_comment_failed",
    "Project item comment could not be saved.",
    request,
  );
}

export function deleteProjectItemCommentFromCookie(
  cookie: string | null | undefined,
  projectId: string,
  itemId: string,
  commentId: string,
): Promise<ProjectItemDetail> {
  return mutateProjectItemDetailFromCookie(
    cookie,
    projectItemCommentPath(projectId, itemId, commentId),
    "DELETE",
    "project_item_comment_failed",
    "Project item comment could not be deleted.",
  );
}

export async function getPersonalAccessTokenListFromCookie(
  cookie: string | null | undefined,
): Promise<PersonalAccessTokenListFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/settings/tokens`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Personal access tokens are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Personal access tokens are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    list: (await response.json()) as PersonalAccessTokenList,
  };
}

export async function getPersonalAccessTokenNewContextFromCookie(
  cookie: string | null | undefined,
): Promise<PersonalAccessTokenNewContextFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/settings/tokens/new`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Token creation is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Token creation is unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    context: (await response.json()) as PersonalAccessTokenNewContext,
  };
}

export async function createPersonalAccessTokenFromCookie(
  cookie: string | null | undefined,
  input: CreatePersonalAccessTokenRequest,
): Promise<CreatePersonalAccessTokenResponse> {
  const response = await fetch(`${apiBaseUrl()}/api/settings/tokens`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Personal access token could not be created.",
      { cause: envelope },
    );
  }

  return (await response.json()) as CreatePersonalAccessTokenResponse;
}

export async function revokePersonalAccessTokenFromCookie(
  cookie: string | null | undefined,
  tokenId: string,
): Promise<RevokePersonalAccessTokenResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/tokens/${encodeURIComponent(tokenId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Personal access token could not be revoked.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RevokePersonalAccessTokenResponse;
}

export async function createSudoGrantFromCookie(
  cookie: string | null | undefined,
  input: { confirmation: string },
): Promise<{ sudo: PersonalAccessTokenSudoState }> {
  const response = await fetch(`${apiBaseUrl()}/api/settings/sudo`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Sudo mode could not be enabled.",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as { sudo: PersonalAccessTokenSudoState };
}

export async function getAccountSecuritySettingsFromCookie(
  cookie: string | null | undefined,
): Promise<AccountSecuritySettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/settings/security`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Account security settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Account security settings are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      // keep fallback
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    settings: (await response.json()) as AccountSecuritySettings,
  };
}

export async function getAccountSessionsFromCookie(
  cookie: string | null | undefined,
): Promise<AccountSessionsFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/settings/security/sessions`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Active sessions are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Active sessions are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      // keep fallback
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    sessions: (await response.json()) as AccountSessions,
  };
}

export async function getAccountSecurityLogFromCookie(
  cookie: string | null | undefined,
  query: AccountSecurityLogQuery = {},
): Promise<AccountSecurityLogFetchResult> {
  const params = new URLSearchParams();
  if (query.action?.trim()) {
    params.set("action", query.action.trim());
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  params.set("pageSize", "50");
  const suffix = params.toString();

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/settings/security-log${suffix ? `?${suffix}` : ""}`,
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
      message: "Security log is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Security log is unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      // keep fallback
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    log: (await response.json()) as AccountSecurityLog,
  };
}

export async function revokeAccountSessionFromCookie(
  cookie: string | null | undefined,
  sessionId: string,
): Promise<{ revokedId: string; sessions: AccountSessions }> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/security/sessions/${encodeURIComponent(
      sessionId,
    )}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Session could not be revoked.",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as {
    revokedId: string;
    sessions: AccountSessions;
  };
}

export async function signOutEverywhereFromCookie(
  cookie: string | null | undefined,
): Promise<{ revokedCount: number; sessions: AccountSessions }> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/security/sessions/sign-out-everywhere`,
    {
      method: "POST",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Other sessions could not be signed out.",
      { cause: envelope },
    );
  }

  return (await response.json()) as {
    revokedCount: number;
    sessions: AccountSessions;
  };
}

export async function createAccountSecuritySudoFromCookie(
  cookie: string | null | undefined,
  input: { confirmation: string },
): Promise<AccountSecuritySettings> {
  const response = await fetch(`${apiBaseUrl()}/api/settings/security/sudo`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Sudo mode could not be enabled.",
      { cause: envelope },
    );
  }

  return (await response.json()) as AccountSecuritySettings;
}

export async function unlinkSignInMethodFromCookie(
  cookie: string | null | undefined,
  accountId: string,
): Promise<UnlinkSignInMethodResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/security/sign-in-methods/${encodeURIComponent(
      accountId,
    )}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Sign-in method could not be unlinked.",
      { cause: envelope },
    );
  }

  return (await response.json()) as UnlinkSignInMethodResponse;
}

export async function getKeySettingsFromCookie(
  cookie: string | null | undefined,
): Promise<KeySettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/settings/keys`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "api_unavailable",
      message: "Signing key settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Signing key settings are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    settings: (await response.json()) as KeySettings,
  };
}

export async function createSshKeyFromCookie(
  cookie: string | null | undefined,
  input: CreateSshKeyRequest,
): Promise<CreateSshKeyResponse> {
  const response = await fetch(`${apiBaseUrl()}/api/settings/keys/ssh`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(envelope?.error.message ?? "SSH key could not be added.", {
      cause: envelope,
    });
  }

  return (await response.json()) as CreateSshKeyResponse;
}

export async function revokeSshKeyFromCookie(
  cookie: string | null | undefined,
  keyId: string,
): Promise<RevokeSshKeyResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/keys/ssh/${encodeURIComponent(keyId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "SSH key could not be deleted.",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as RevokeSshKeyResponse;
}

export async function createGpgKeyFromCookie(
  cookie: string | null | undefined,
  input: CreateGpgKeyRequest,
): Promise<CreateGpgKeyResponse> {
  const response = await fetch(`${apiBaseUrl()}/api/settings/keys/gpg`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(envelope?.error.message ?? "GPG key could not be added.", {
      cause: envelope,
    });
  }

  return (await response.json()) as CreateGpgKeyResponse;
}

export async function revokeGpgKeyFromCookie(
  cookie: string | null | undefined,
  keyId: string,
): Promise<RevokeGpgKeyResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/keys/gpg/${encodeURIComponent(keyId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "GPG key could not be deleted.",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as RevokeGpgKeyResponse;
}

export async function updateVigilantModeFromCookie(
  cookie: string | null | undefined,
  input: UpdateVigilantModeRequest,
): Promise<UpdateVigilantModeResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/settings/keys/vigilant-mode`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let envelope: ApiErrorEnvelope | null = null;
    try {
      envelope = (await response.json()) as ApiErrorEnvelope;
    } catch {
      envelope = null;
    }
    throw new Error(
      envelope?.error.message ?? "Vigilant mode could not be updated.",
      {
        cause: envelope,
      },
    );
  }

  return (await response.json()) as UpdateVigilantModeResponse;
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

export async function getPersonalProfileSettingsFromCookie(
  cookie: string | null | undefined,
): Promise<PersonalProfileSettings | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/user/settings/profile`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as PersonalProfileSettings;
}

export async function getAppearanceSettingsFromCookie(
  cookie: string | null | undefined,
): Promise<AppearanceSettings | null> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/user/settings/appearance`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return null;
  }

  if (!response.ok) {
    return null;
  }

  return (await response.json()) as AppearanceSettings;
}

export async function updateAppearanceSettingsFromCookie(
  cookie: string | null | undefined,
  input: UpdateAppearanceSettingsRequest,
): Promise<AppearanceSettings> {
  const response = await fetch(`${apiBaseUrl()}/api/user/settings/appearance`, {
    method: "PATCH",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      body?.error.message ?? "Appearance settings update failed",
      {
        cause: body,
      },
    );
  }

  return (await response.json()) as AppearanceSettings;
}

export async function updatePersonalProfileSettingsFromCookie(
  cookie: string | null | undefined,
  input: UpdatePersonalProfileSettingsRequest,
): Promise<PersonalProfileSettings> {
  const response = await fetch(`${apiBaseUrl()}/api/user/settings/profile`, {
    method: "PATCH",
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Profile settings update failed", {
      cause: body,
    });
  }

  return (await response.json()) as PersonalProfileSettings;
}

export async function updatePersonalAvatarFromCookie(
  cookie: string | null | undefined,
  input: UpdateAvatarRequest,
): Promise<PersonalProfileSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/user/settings/profile/avatar`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Avatar update failed", {
      cause: body,
    });
  }

  return (await response.json()) as PersonalProfileSettings;
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

export async function getOrganizationProfileSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
): Promise<OrganizationProfileSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/profile`,
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
      message: "Organization settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Organization settings are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    settings: (await response.json()) as OrganizationProfileSettings,
  };
}

export async function getOrganizationMemberPrivilegesFromCookie(
  cookie: string | null | undefined,
  org: string,
): Promise<OrganizationMemberPrivilegesFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/member-privileges`,
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
      message: "Organization member privileges are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Organization member privileges are unavailable right now.";
    try {
      const body = (await response.json()) as ApiErrorEnvelope;
      code = body.error.code ?? null;
      message = body.error.message ?? message;
    } catch {
      code = null;
    }
    return { ok: false, status: response.status, code, message };
  }

  return {
    ok: true,
    settings: (await response.json()) as OrganizationMemberPrivilegesSettings,
  };
}

export async function updateOrganizationProfileSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  input: UpdateOrganizationProfileSettingsRequest,
): Promise<OrganizationProfileSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/profile`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      body?.error.message ?? "Organization profile settings update failed",
      { cause: body },
    );
  }

  return (await response.json()) as OrganizationProfileSettings;
}

export async function updateOrganizationMemberPrivilegesFromCookie(
  cookie: string | null | undefined,
  org: string,
  input: UpdateOrganizationMemberPrivilegesRequest,
): Promise<OrganizationMemberPrivilegesSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/member-privileges`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      body?.error.message ?? "Organization member privileges update failed",
      { cause: body },
    );
  }

  return (await response.json()) as OrganizationMemberPrivilegesSettings;
}

export async function renameOrganizationFromCookie(
  cookie: string | null | undefined,
  org: string,
  input: RenameOrganizationRequest,
): Promise<OrganizationProfileSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/profile/rename`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Organization rename failed", {
      cause: body,
    });
  }

  return (await response.json()) as OrganizationProfileSettings;
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

export async function getOrganizationPeopleAdminFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: OrganizationPeopleAdminQuery = {},
): Promise<OrganizationPeopleAdmin | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people/admin`,
    );
    if (query.tab) {
      url.searchParams.set("tab", query.tab);
    }
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

  return (await response.json()) as OrganizationPeopleAdmin;
}

export async function getOrganizationTeamsFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: OrganizationTeamsQuery = {},
): Promise<OrganizationTeamsDirectory | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/teams`,
    );
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.visibility) {
      url.searchParams.set("visibility", query.visibility);
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

  return (await response.json()) as OrganizationTeamsDirectory;
}

export async function createOrganizationTeamFromCookie(
  cookie: string | null | undefined,
  org: string,
  input: CreateOrganizationTeamRequest,
): Promise<OrganizationTeamCreateResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/teams`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Team creation failed", {
      cause: envelope,
    });
  }

  return (await response.json()) as OrganizationTeamCreateResult;
}

export async function getOrganizationTeamDetailFromCookie(
  cookie: string | null | undefined,
  org: string,
  teamSlug: string,
): Promise<OrganizationTeamDetail | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/teams/${encodeURIComponent(teamSlug)}`,
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

  return (await response.json()) as OrganizationTeamDetail;
}

export async function mutateOrganizationPeopleAdminFromCookie(
  cookie: string | null | undefined,
  org: string,
  mutation: OrganizationInvitationMutation,
): Promise<OrganizationPeopleAdmin> {
  let path = `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people/invitations`;
  let method = "POST";
  let body: unknown;

  if (mutation.action === "invite") {
    body = {
      emailOrLogin: mutation.emailOrLogin,
      role: mutation.role,
      teamIds: mutation.teamIds ?? [],
    };
  } else if (mutation.action === "retry") {
    path = `${path}/${encodeURIComponent(mutation.invitationId)}/retry`;
  } else if (mutation.action === "cancel") {
    path = `${path}/${encodeURIComponent(mutation.invitationId)}`;
    method = "DELETE";
  } else if (mutation.action === "visibility") {
    path = `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people/members/${encodeURIComponent(mutation.userId)}/visibility`;
    method = "PATCH";
    body = { visibility: mutation.visibility };
  } else if (mutation.action === "role") {
    path = `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people/members/${encodeURIComponent(mutation.userId)}/role`;
    method = "PATCH";
    body = { role: mutation.role };
  } else {
    path = `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/people/members/${encodeURIComponent(mutation.userId)}`;
    method = "DELETE";
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Organization people update failed",
      { cause: envelope },
    );
  }

  return (await response.json()) as OrganizationPeopleAdmin;
}

export async function getUserPackagesFromCookie(
  cookie: string | null | undefined,
  username: string,
  query: OwnerPackageListQuery = {},
): Promise<OwnerPackageList | null> {
  return getOwnerPackagesFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/packages`,
    query,
  );
}

export async function getOrganizationPackagesFromCookie(
  cookie: string | null | undefined,
  org: string,
  query: OwnerPackageListQuery = {},
): Promise<OwnerPackageList | null> {
  return getOwnerPackagesFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/packages`,
    query,
  );
}

export async function getUserPackageDetailFromCookie(
  cookie: string | null | undefined,
  username: string,
  packageType: string,
  packageName: string,
  version?: string | null,
): Promise<PackageDetailFetchResult> {
  return getPackageDetailFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}`,
    version,
  );
}

export async function getUserPackageSettingsFromCookie(
  cookie: string | null | undefined,
  username: string,
  packageType: string,
  packageName: string,
): Promise<PackageSettingsFetchResult> {
  return getPackageSettingsFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}/settings`,
  );
}

export async function getOrganizationPackageDetailFromCookie(
  cookie: string | null | undefined,
  org: string,
  packageType: string,
  packageName: string,
  version?: string | null,
): Promise<PackageDetailFetchResult> {
  return getPackageDetailFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}`,
    version,
  );
}

export async function getOrganizationPackageSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  packageType: string,
  packageName: string,
): Promise<PackageSettingsFetchResult> {
  return getPackageSettingsFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}/settings`,
  );
}

export async function mutateUserPackageSettingsFromCookie(
  cookie: string | null | undefined,
  username: string,
  packageType: string,
  packageName: string,
  mutation: PackageSettingsMutation,
): Promise<PackageSettings> {
  return mutatePackageSettingsFromCookie(
    cookie,
    `/api/users/${encodeURIComponent(username)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}/settings`,
    mutation,
  );
}

export async function mutateOrganizationPackageSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  packageType: string,
  packageName: string,
  mutation: PackageSettingsMutation,
): Promise<PackageSettings> {
  return mutatePackageSettingsFromCookie(
    cookie,
    `/api/orgs/${encodeURIComponent(org)}/packages/${encodeURIComponent(packageType)}/${encodeURIComponent(packageName)}/settings`,
    mutation,
  );
}

async function getPackageDetailFromCookie(
  cookie: string | null | undefined,
  path: string,
  version?: string | null,
): Promise<PackageDetailFetchResult> {
  let response: Response;
  try {
    const url = new URL(`${apiBaseUrl()}${path}`);
    if (version?.trim()) {
      url.searchParams.set("version", version.trim());
    }
    response = await fetch(url, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "packages_unavailable",
      message: "Package detail could not be reached.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Package detail could not be loaded.",
    };
  }

  return { ok: true, package: (await response.json()) as PackageDetail };
}

async function getPackageSettingsFromCookie(
  cookie: string | null | undefined,
  path: string,
): Promise<PackageSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${path}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      ok: false,
      status: 503,
      code: "packages_unavailable",
      message: "Package settings could not be reached.",
    };
  }

  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    return {
      ok: false,
      status: body?.status ?? response.status,
      code: body?.error.code ?? null,
      message: body?.error.message ?? "Package settings could not be loaded.",
    };
  }

  return { ok: true, settings: (await response.json()) as PackageSettings };
}

async function mutatePackageSettingsFromCookie(
  cookie: string | null | undefined,
  path: string,
  mutation: PackageSettingsMutation,
): Promise<PackageSettings> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(mutation),
  });
  if (!response.ok) {
    let body: ApiErrorEnvelope | null = null;
    try {
      body = (await response.json()) as ApiErrorEnvelope;
    } catch {
      body = null;
    }
    throw new Error(body?.error.message ?? "Package settings update failed.", {
      cause: body,
    });
  }
  return (await response.json()) as PackageSettings;
}

async function getOwnerPackagesFromCookie(
  cookie: string | null | undefined,
  path: string,
  query: OwnerPackageListQuery,
): Promise<OwnerPackageList | null> {
  let response: Response;
  try {
    const url = new URL(`${apiBaseUrl()}${path}`);
    if (query.q) {
      url.searchParams.set("q", query.q);
    }
    if (query.type) {
      url.searchParams.set("type", query.type);
    }
    if (query.visibility) {
      url.searchParams.set("visibility", query.visibility);
    }
    if (query.sort) {
      url.searchParams.set("sort", query.sort);
    }
    if (query.artifactTab) {
      url.searchParams.set("artifactTab", query.artifactTab);
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

  return (await response.json()) as OwnerPackageList;
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

export async function getProfileSocialListFromCookie(
  cookie: string | null | undefined,
  username: string,
  mode: "followers" | "following",
  query: Pick<ProfileRepositoryListQuery, "page" | "pageSize"> = {},
): Promise<ProfileSocialList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/users/${encodeURIComponent(username)}/${mode}`,
    );
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

  return (await response.json()) as ProfileSocialList;
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

export async function getSearchIndexStatusFromCookie(
  cookie: string | null | undefined,
): Promise<SearchIndexStatus | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/admin/search`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Search index status is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "search_index_status_failed",
          message: "Search index status could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as SearchIndexStatus;
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

export function globalIssuesPath(query: GlobalIssueListQuery = {}): string {
  const params = new URLSearchParams();
  if (query.scope) {
    params.set("scope", query.scope);
  }
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state) {
    params.set("state", query.state);
  }
  const repository = query.repository ?? query.repo;
  if (repository?.trim()) {
    params.set("repo", repository.trim());
  }
  if (query.labels?.length) {
    params.set("labels", query.labels.join(","));
  }
  if (query.milestone?.trim()) {
    params.set("milestone", query.milestone.trim());
  }
  if (query.project?.trim()) {
    params.set("project", query.project.trim());
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
  return `/api/issues${suffix}`;
}

export async function getGlobalIssuesFromCookie(
  cookie: string | null | undefined,
  query: GlobalIssueListQuery = {},
): Promise<GlobalIssueListView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${globalIssuesPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
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

  return body as GlobalIssueListView;
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

export function repositoryMilestonesPath(
  owner: string,
  repo: string,
  query: RepositoryMilestoneListQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.state) {
    params.set("state", query.state);
  }
  if (query.sort) {
    params.set("sort", query.sort);
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones${suffix}`;
}

export function repositoryMilestonePath(
  owner: string,
  repo: string,
  milestoneId: string,
): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/${encodeURIComponent(milestoneId)}`;
}

export async function getRepositoryMilestonesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryMilestoneListQuery = {},
): Promise<RepositoryMilestonesView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryMilestonesPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Milestones are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "milestones_failed",
          message: "Milestones could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryMilestonesView;
}

export async function getRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
): Promise<RepositoryMilestoneDetail | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryMilestonePath(owner, repo, milestoneId)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Milestone is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "milestone_failed",
          message: "Milestone could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryMilestoneDetail;
}

async function mutateRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH" | "DELETE",
  body?: unknown,
): Promise<RepositoryMilestoneDetail | null> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      ...(body === undefined ? {} : { "content-type": "application/json" }),
      ...(cookie ? { cookie } : {}),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Milestone operation could not be completed",
      { cause: envelope },
    );
  }

  if (response.status === 204) {
    return null;
  }

  return (await response.json()) as RepositoryMilestoneDetail;
}

export async function createRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  input: RepositoryMilestoneMutation,
): Promise<RepositoryMilestoneDetail> {
  const result = await mutateRepositoryMilestoneFromCookie(
    cookie,
    repositoryMilestonesPath(owner, repo),
    "POST",
    input,
  );
  if (!result) {
    throw new Error("Milestone create returned no milestone");
  }
  return result;
}

export async function updateRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
  input: RepositoryMilestoneMutation,
): Promise<RepositoryMilestoneDetail> {
  const result = await mutateRepositoryMilestoneFromCookie(
    cookie,
    repositoryMilestonePath(owner, repo, milestoneId),
    "PATCH",
    input,
  );
  if (!result) {
    throw new Error("Milestone update returned no milestone");
  }
  return result;
}

export async function closeRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
): Promise<RepositoryMilestoneDetail> {
  const result = await mutateRepositoryMilestoneFromCookie(
    cookie,
    `${repositoryMilestonePath(owner, repo, milestoneId)}/close`,
    "POST",
    { state: "closed" },
  );
  if (!result) {
    throw new Error("Milestone close returned no milestone");
  }
  return result;
}

export async function reopenRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
): Promise<RepositoryMilestoneDetail> {
  const result = await mutateRepositoryMilestoneFromCookie(
    cookie,
    `${repositoryMilestonePath(owner, repo, milestoneId)}/reopen`,
    "POST",
    { state: "open" },
  );
  if (!result) {
    throw new Error("Milestone reopen returned no milestone");
  }
  return result;
}

export async function deleteRepositoryMilestoneFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
): Promise<void> {
  await mutateRepositoryMilestoneFromCookie(
    cookie,
    repositoryMilestonePath(owner, repo, milestoneId),
    "DELETE",
  );
}

export async function reorderRepositoryMilestoneItemsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  milestoneId: string,
  input: RepositoryMilestoneOrderRequest,
): Promise<RepositoryMilestoneDetail> {
  const result = await mutateRepositoryMilestoneFromCookie(
    cookie,
    `${repositoryMilestonePath(owner, repo, milestoneId)}/order`,
    "PATCH",
    input,
  );
  if (!result) {
    throw new Error("Milestone reorder returned no milestone");
  }
  return result;
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

export function globalPullRequestsPath(
  query: GlobalPullRequestListQuery = {},
): string {
  const params = new URLSearchParams();
  if (query.scope) {
    params.set("scope", query.scope);
  }
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state) {
    params.set("state", query.state);
  }
  const repository = query.repository ?? query.repo;
  if (repository?.trim()) {
    params.set("repo", repository.trim());
  }
  if (query.labels?.length) {
    params.set("labels", query.labels.join(","));
  }
  if (query.milestone?.trim()) {
    params.set("milestone", query.milestone.trim());
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
  return `/api/pulls${suffix}`;
}

export async function getGlobalPullRequestsFromCookie(
  cookie: string | null | undefined,
  query: GlobalPullRequestListQuery = {},
): Promise<GlobalPullRequestListView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${globalPullRequestsPath(query)}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
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

  return body as GlobalPullRequestListView;
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

export function repositoryActionsCachesPath(
  owner: string,
  repo: string,
  query: { page?: number | null; pageSize?: number | null } = {},
): string {
  const params = new URLSearchParams();
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(
    repo,
  )}/actions/caches${suffix}`;
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

export async function getRepositoryActionsCachesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: { page?: number | null; pageSize?: number | null } = {},
): Promise<RepositoryActionsCaches | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryActionsCachesPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Actions caches are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "actions_caches_failed",
          message: "Actions caches could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryActionsCaches;
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

export async function getRepositoryPullRequestChecksFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
): Promise<PullRequestChecksView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/checks`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Pull request checks are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "pull_request_checks_failed",
          message: "Pull request checks could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as PullRequestChecksView;
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
  customEvents: string[] = [],
): Promise<PullRequestSubscriptionState> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryPullRequestPath(owner, repo, number)}/subscription`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ subscribed, customEvents }),
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
  customEvents: string[] = [],
): Promise<IssueSubscriptionState> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/subscription`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ subscribed, customEvents }),
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

export async function getIssueDiscussionConversionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
): Promise<IssueDiscussionConversionView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/convert-to-discussion`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion conversion metadata failed to load",
      { cause: envelope },
    );
  }

  return (await response.json()) as IssueDiscussionConversionView;
}

export async function convertIssueToDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  issueNumber: number | string,
  categorySlug: string,
): Promise<ConvertIssueToDiscussionResponse> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryIssuePath(owner, repo, issueNumber)}/convert-to-discussion`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ categorySlug }),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Issue could not be converted to a discussion",
      { cause: envelope },
    );
  }

  return (await response.json()) as ConvertIssueToDiscussionResponse;
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

function aiEndpoint(
  owner: string,
  repo: string,
  suffix: string,
  regenerate = false,
) {
  const query = regenerate ? "?regenerate=true" : "";
  return `/api/ai/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}${suffix}${query}`;
}

async function fetchAiFromCookie<T>(
  cookie: string | null | undefined,
  endpoint: string,
  fallbackMessage: string,
  init?: RequestInit,
): Promise<T | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${endpoint}`, {
      ...init,
      headers: {
        ...(init?.headers ?? {}),
        ...(cookie ? { cookie } : {}),
      },
      cache: "no-store",
    });
  } catch {
    return {
      error: { code: "network_error", message: fallbackMessage },
      status: 503,
    };
  }
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (payload as ApiErrorEnvelope | null) ?? {
        error: { code: "ai_failed", message: fallbackMessage },
        status: response.status,
      }
    );
  }
  return payload as T;
}

export function getRepositoryAiSummaryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
) {
  return fetchAiFromCookie<RepositoryAiSummary>(
    cookie,
    aiEndpoint(owner, repo, "/summary"),
    "Repository AI summary is temporarily unavailable.",
  );
}

export function regenerateRepositoryAiSummaryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
) {
  return fetchAiFromCookie<RepositoryAiSummary>(
    cookie,
    aiEndpoint(owner, repo, "/summary", true),
    "Repository AI summary could not be regenerated.",
    { method: "POST" },
  );
}

export function getPullRequestAiSummaryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
) {
  return fetchAiFromCookie<PullRequestAiSummary>(
    cookie,
    aiEndpoint(
      owner,
      repo,
      `/pulls/${encodeURIComponent(String(number))}/summary`,
    ),
    "Pull request AI summary is temporarily unavailable.",
  );
}

export function regeneratePullRequestAiSummaryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  number: number | string,
) {
  return fetchAiFromCookie<PullRequestAiSummary>(
    cookie,
    aiEndpoint(
      owner,
      repo,
      `/pulls/${encodeURIComponent(String(number))}/summary`,
      true,
    ),
    "Pull request AI summary could not be regenerated.",
    { method: "POST" },
  );
}

export function generateAiChangelogFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: AiChangelogRequest,
) {
  return fetchAiFromCookie<AiChangelog>(
    cookie,
    aiEndpoint(owner, repo, "/releases/changelog", true),
    "AI changelog could not be generated.",
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(request),
    },
  );
}

function repositoryLabelsPath(
  owner: string,
  repo: string,
  query: RepositoryLabelsQuery = {},
) {
  const params = new URLSearchParams();
  if (query.q) params.set("q", query.q);
  if (query.sort) params.set("sort", query.sort);
  if (query.direction) params.set("direction", query.direction);
  if (query.page) params.set("page", String(query.page));
  if (query.pageSize) params.set("pageSize", String(query.pageSize));
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels${suffix}`;
}

export async function getRepositoryLabelsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryLabelsQuery = {},
): Promise<RepositoryLabelsView> {
  const response = await fetch(
    `${apiBaseUrl()}${repositoryLabelsPath(owner, repo, query)}`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Repository labels failed.", {
      cause: envelope,
    });
  }
  return payload as RepositoryLabelsView;
}

export async function createRepositoryLabelFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: RepositoryLabelMutationRequest,
): Promise<RepositoryLabelMutationResult> {
  return mutateRepositoryLabelFromCookie(
    cookie,
    owner,
    repo,
    null,
    "POST",
    request,
  );
}

export async function updateRepositoryLabelFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  labelId: string,
  request: RepositoryLabelMutationRequest,
): Promise<RepositoryLabelMutationResult> {
  return mutateRepositoryLabelFromCookie(
    cookie,
    owner,
    repo,
    labelId,
    "PATCH",
    request,
  );
}

export async function deleteRepositoryLabelFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  labelId: string,
): Promise<RepositoryLabelMutationResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels/${encodeURIComponent(labelId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository label delete failed.",
      {
        cause: envelope,
      },
    );
  }
  return payload as RepositoryLabelMutationResult;
}

async function mutateRepositoryLabelFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  labelId: string | null,
  method: "POST" | "PATCH",
  request: RepositoryLabelMutationRequest,
): Promise<RepositoryLabelMutationResult> {
  const path = labelId
    ? `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels/${encodeURIComponent(labelId)}`
    : repositoryLabelsPath(owner, repo);
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: JSON.stringify(request),
    cache: "no-store",
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository label save failed.",
      {
        cause: envelope,
      },
    );
  }
  return payload as RepositoryLabelMutationResult;
}

function repositoryReleaseListPath(
  owner: string,
  repo: string,
  query: RepositoryReleaseListQuery = {},
) {
  const params = new URLSearchParams();
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases${suffix}`;
}

function repositoryReleaseError(
  body: unknown,
  status: number,
  message: string,
): ApiErrorEnvelope {
  return (
    (body as ApiErrorEnvelope | null) ?? {
      error: { code: "releases_failed", message },
      status,
    }
  );
}

export async function getRepositoryReleasesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryReleaseListQuery = {},
): Promise<ListEnvelope<RepositoryReleaseSummary> | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${repositoryReleaseListPath(owner, repo, query)}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Repository releases are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return repositoryReleaseError(
      body,
      response.status,
      "Repository releases could not be loaded.",
    );
  }
  return body as ListEnvelope<RepositoryReleaseSummary>;
}

export async function getRepositoryReleaseDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  tag: string,
): Promise<RepositoryReleaseDetail | ApiErrorEnvelope> {
  const endpoint =
    tag === "latest"
      ? `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/latest`
      : `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/tag/${encodeURIComponent(tag)}`;
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${endpoint}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Repository release is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return repositoryReleaseError(
      body,
      response.status,
      "Repository release could not be loaded.",
    );
  }
  return body as RepositoryReleaseDetail;
}

export async function getRepositoryReleaseTagsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryReleaseListQuery = {},
): Promise<ListEnvelope<ReleaseTagSummary> | ApiErrorEnvelope> {
  const params = new URLSearchParams();
  if (query.page) {
    params.set("page", String(query.page));
  }
  if (query.pageSize) {
    params.set("pageSize", String(query.pageSize));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/tags${suffix}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Repository tags are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return repositoryReleaseError(
      body,
      response.status,
      "Repository tags could not be loaded.",
    );
  }
  return body as ListEnvelope<ReleaseTagSummary>;
}

export async function getRepositoryReleaseManagementContextFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId?: string | null,
): Promise<ReleaseManagementContext | ApiErrorEnvelope> {
  const suffix = releaseId ? `/${encodeURIComponent(releaseId)}` : "";
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/manage${suffix}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Release management is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return repositoryReleaseError(
      body,
      response.status,
      "Release management could not be loaded.",
    );
  }
  return body as ReleaseManagementContext;
}

export async function toggleRepositoryReleaseReactionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
  content: ReleaseReactionContent,
): Promise<ReleaseReactionSummary> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}/reactions`,
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
      envelope?.error.message ?? "Release reaction could not be updated",
      {
        cause: envelope,
      },
    );
  }

  return payload as ReleaseReactionSummary;
}

export async function generateRepositoryReleaseNotesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: GeneratedReleaseNotesRequest,
): Promise<GeneratedReleaseNotesPreview> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/manage/generated-notes`,
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
      envelope?.error.message ?? "Release notes could not be generated",
      {
        cause: envelope,
      },
    );
  }

  return payload as GeneratedReleaseNotesPreview;
}

export async function getRepositoryReleaseAssetDownloadFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  assetId: string,
): Promise<ReleaseAssetDownloadMetadata> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/assets/${encodeURIComponent(assetId)}`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Release asset could not load", {
      cause: envelope,
    });
  }

  return payload as ReleaseAssetDownloadMetadata;
}

async function releaseMutationRequest(
  cookie: string | null | undefined,
  endpoint: string,
  init: RequestInit,
): Promise<RepositoryReleaseDetail> {
  const response = await fetch(`${apiBaseUrl()}${endpoint}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
      ...init.headers,
    },
    cache: "no-store",
  });
  if (response.status === 204) {
    return {} as RepositoryReleaseDetail;
  }
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Release could not be updated", {
      cause: envelope,
    });
  }
  return payload as RepositoryReleaseDetail;
}

export async function createRepositoryReleaseFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: ReleaseMutation,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases`,
    {
      method: "POST",
      body: JSON.stringify(mutation),
    },
  );
}

export async function updateRepositoryReleaseFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
  mutation: ReleaseMutation,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}`,
    {
      method: "PATCH",
      body: JSON.stringify(mutation),
    },
  );
}

export async function publishRepositoryReleaseFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}/publish`,
    {
      method: "POST",
      body: "{}",
    },
  );
}

export async function deleteRepositoryReleaseFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
  mutation: Pick<ReleaseMutation, "deleteTag"> = {},
): Promise<void> {
  await releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}`,
    {
      method: "DELETE",
      body: JSON.stringify(mutation),
    },
  );
}

export async function createRepositoryReleaseAssetFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
  mutation: ReleaseAssetMutation,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}/assets`,
    {
      method: "POST",
      body: JSON.stringify(mutation),
    },
  );
}

export async function createRepositoryReleaseUploadIntentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: ReleaseUploadIntentRequest,
): Promise<ReleaseUploadIntent> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/manage/upload-intents`,
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
      envelope?.error.message ?? "Release asset upload could not be started",
      {
        cause: envelope,
      },
    );
  }
  return payload as ReleaseUploadIntent;
}

export async function completeRepositoryReleaseUploadIntentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  intentId: string,
  request: ReleaseUploadCompleteRequest,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/manage/upload-intents/${encodeURIComponent(intentId)}/complete`,
    {
      method: "POST",
      body: JSON.stringify(request),
    },
  );
}

export async function cancelRepositoryReleaseUploadIntentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  intentId: string,
  reason?: string,
): Promise<ReleaseUploadIntent> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/manage/upload-intents/${encodeURIComponent(intentId)}/cancel`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ reason: reason ?? "cancelled by user" }),
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Release asset upload could not be cancelled",
      {
        cause: envelope,
      },
    );
  }
  return payload as ReleaseUploadIntent;
}

export async function deleteRepositoryReleaseAssetFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  releaseId: string,
  assetId: string,
): Promise<RepositoryReleaseDetail> {
  return releaseMutationRequest(
    cookie,
    `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/${encodeURIComponent(releaseId)}/assets/${encodeURIComponent(assetId)}`,
    {
      method: "DELETE",
      body: "{}",
    },
  );
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

export async function getRepositoryAccessSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryAccessSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/access`,
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
      message: "Repository access settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository access settings are unavailable right now.";
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
    settings: (await response.json()) as RepositoryAccessSettings,
  };
}

export async function getRepositoryBranchSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryBranchSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/branches`,
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
      message: "Repository branch settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository branch settings are unavailable right now.";
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
    settings: (await response.json()) as RepositoryBranchSettings,
  };
}

export async function getRepositoryBranchesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: {
    tab?: string | null;
    query?: string | null;
    page?: number | null;
    pageSize?: number | null;
  } = {},
): Promise<RepositoryBranchesFetchResult> {
  const params = new URLSearchParams();
  if (options.tab?.trim() && options.tab.trim() !== "overview") {
    params.set("tab", options.tab.trim());
  }
  if (options.query?.trim()) {
    params.set("q", options.query.trim());
  }
  if (options.page && Number.isFinite(options.page) && options.page > 1) {
    params.set("page", String(options.page));
  }
  if (
    options.pageSize &&
    Number.isFinite(options.pageSize) &&
    options.pageSize !== 30
  ) {
    params.set("pageSize", String(options.pageSize));
  }
  const query = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches${query ? `?${query}` : ""}`,
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
      message: "Repository branches are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository branches are unavailable right now.";
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
    branches: (await response.json()) as RepositoryBranchesView,
  };
}

export async function getRepositoryBranchActivityFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  branch: string,
): Promise<RepositoryBranchActivityFetchResult> {
  const params = new URLSearchParams({ branch });
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches/activity?${params.toString()}`,
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
      message: "Repository branch activity is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository branch activity is unavailable right now.";
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
    activity: (await response.json()) as RepositoryBranchActivityView,
  };
}

export async function getRepositoryPulseFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: { period?: string | null } = {},
): Promise<RepositoryPulseFetchResult> {
  const params = new URLSearchParams();
  if (options.period?.trim()) {
    params.set("period", options.period.trim());
  }
  const query = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulse${query ? `?${query}` : ""}`,
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
      message: "Repository Pulse is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Pulse is unavailable right now.";
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
    pulse: (await response.json()) as RepositoryPulseView,
  };
}

export async function getRepositoryContributorsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: {
    period?: string | null;
    start?: string | null;
    end?: string | null;
  } = {},
): Promise<RepositoryContributorsFetchResult> {
  const params = new URLSearchParams();
  if (options.period?.trim()) {
    params.set("period", options.period.trim());
  }
  if (options.start?.trim()) {
    params.set("start", options.start.trim());
  }
  if (options.end?.trim()) {
    params.set("end", options.end.trim());
  }
  const query = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/graphs/contributors${query ? `?${query}` : ""}`,
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
      message: "Repository Contributors is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Contributors is unavailable right now.";
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
    contributors: (await response.json()) as RepositoryContributorsView,
  };
}

export async function getRepositoryTrafficFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryTrafficFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/graphs/traffic`,
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
      message: "Repository Traffic is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Traffic is unavailable right now.";
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
    traffic: (await response.json()) as RepositoryTrafficView,
  };
}

export async function getRepositorySecurityOverviewFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositorySecurityOverviewFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security`,
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
      message: "Repository Security is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Security is unavailable right now.";
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
    security: (await response.json()) as RepositorySecurityOverviewView,
  };
}

export async function getRepositorySecurityPolicyFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositorySecurityPolicyFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/policy`,
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
      message: "Repository Security policy is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Security policy is unavailable right now.";
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
    securityPolicy: (await response.json()) as RepositorySecurityPolicyView,
  };
}

export async function getRepositoryWikiFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  slug?: string | null,
): Promise<RepositoryWikiFetchResult> {
  const encodedSlug = slug
    ? `/${slug
        .split("/")
        .filter(Boolean)
        .map((segment) => encodeURIComponent(segment))
        .join("/")}`
    : "";
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki${encodedSlug}`,
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
      message: "Repository wiki is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository wiki is unavailable right now.";
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
    wiki: (await response.json()) as RepositoryWikiView,
  };
}

export async function getRepositoryWikiHistoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  slug?: string | null,
  page?: number | null,
  pageSize?: number | null,
): Promise<RepositoryWikiHistoryFetchResult> {
  const encodedSlug = slug
    ? `/${slug
        .split("/")
        .filter(Boolean)
        .map((segment) => encodeURIComponent(segment))
        .join("/")}`
    : "";
  const params = new URLSearchParams();
  if (page && page > 1) params.set("page", String(page));
  if (pageSize && pageSize !== 30) params.set("pageSize", String(pageSize));
  const query = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki${encodedSlug}/_history${query ? `?${query}` : ""}`,
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
      message: "Repository wiki history is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository wiki history is unavailable right now.";
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
    history: (await response.json()) as RepositoryWikiHistoryView,
  };
}

export async function getRepositoryWikiRevisionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  slug: string,
  revision: string,
): Promise<RepositoryWikiRevisionFetchResult> {
  const encodedSlug = slug
    .split("/")
    .filter(Boolean)
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/${encodedSlug}/_history/${encodeURIComponent(revision)}`,
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
      message: "Repository wiki revision is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository wiki revision is unavailable right now.";
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
    revision: (await response.json()) as RepositoryWikiRevisionView,
  };
}

export async function getRepositoryWikiCompareFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  base: string,
  head: string,
  slug?: string | null,
): Promise<RepositoryWikiCompareFetchResult> {
  const params = new URLSearchParams({ base, head });
  if (slug?.trim()) params.set("page", slug.trim());
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/_compare?${params.toString()}`,
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
      message: "Repository wiki compare is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository wiki compare is unavailable right now.";
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
    compare: (await response.json()) as RepositoryWikiCompareView,
  };
}

export async function getRepositoryWikiPagesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryWikiPagesIndex> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/_pages`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki pages failed to load.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiPagesIndex;
}

export async function getRepositoryWikiEditFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  slug: string,
): Promise<RepositoryWikiEditView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/${encodeURIComponent(slug)}/edit`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki editor failed to load.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiEditView;
}

export async function previewRepositoryWikiFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: RepositoryWikiPreviewRequest,
): Promise<RepositoryWikiPreviewResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/preview`,
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
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki preview failed.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiPreviewResult;
}

export async function createRepositoryWikiPageFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: RepositoryWikiSaveRequest,
): Promise<RepositoryWikiMutationResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/pages`,
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
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki page save failed.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiMutationResult;
}

export async function updateRepositoryWikiPageFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  slug: string,
  request: RepositoryWikiSaveRequest,
): Promise<RepositoryWikiMutationResult> {
  const encodedSlug = slug
    .split("/")
    .filter(Boolean)
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/${encodedSlug}`,
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
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki page save failed.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiMutationResult;
}

export async function revertRepositoryWikiPageFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: RepositoryWikiRevertRequest,
): Promise<RepositoryWikiRevertResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/reverts`,
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
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository wiki revert failed.",
      {
        cause: envelope,
      },
    );
  }
  return (await response.json()) as RepositoryWikiRevertResult;
}

export async function mutateRepositorySecurityPolicyFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositorySecurityPolicyMutation,
  method: "POST" | "PATCH",
): Promise<RepositorySecurityPolicyView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/policy`,
    {
      method,
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository Security policy update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositorySecurityPolicyView;
}

function appendRepositorySecurityAdvisorySearchParams(
  searchParams: URLSearchParams,
  query?: RepositorySecurityAdvisoriesQuery,
) {
  if (!query) {
    return;
  }
  const entries: [string, string | number | null | undefined][] = [
    ["state", query.state],
    ["q", query.query],
    ["severity", query.severity],
    ["sort", query.sort],
    ["page", query.page],
    ["page_size", query.pageSize],
  ];
  for (const [key, value] of entries) {
    if (value !== null && value !== undefined && String(value).trim()) {
      searchParams.set(key, String(value));
    }
  }
}

export async function getRepositorySecurityAdvisoriesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query?: RepositorySecurityAdvisoriesQuery,
): Promise<RepositorySecurityAdvisoriesFetchResult> {
  const searchParams = new URLSearchParams();
  appendRepositorySecurityAdvisorySearchParams(searchParams, query);
  const suffix = searchParams.size ? `?${searchParams.toString()}` : "";

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/advisories${suffix}`,
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
      message: "Repository security advisories are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository security advisories are unavailable right now.";
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
    advisories: (await response.json()) as RepositorySecurityAdvisoriesView,
  };
}

export async function getRepositorySecurityAdvisoryDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  ghsaId: string,
): Promise<RepositorySecurityAdvisoryDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/advisories/${encodeURIComponent(ghsaId)}`,
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
      message: "Repository security advisory detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message =
      "Repository security advisory detail is unavailable right now.";
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
    advisory: (await response.json()) as RepositorySecurityAdvisoryDetail,
  };
}

export async function mutateRepositorySecurityAdvisoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  ghsaId: string,
  mutation: RepositorySecurityAdvisoryMutation,
): Promise<RepositorySecurityAdvisoryDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/advisories/${encodeURIComponent(ghsaId)}`,
    {
      method: "PATCH",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository security advisory update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositorySecurityAdvisoryDetail;
}

export async function createRepositorySecurityAdvisoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: RepositorySecurityAdvisoryCreate,
): Promise<RepositorySecurityAdvisoryDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/advisories`,
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
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Repository security advisory creation failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositorySecurityAdvisoryDetail;
}

export async function publishRepositorySecurityAdvisoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  ghsaId: string,
): Promise<RepositorySecurityAdvisoryDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/advisories/${encodeURIComponent(ghsaId)}/publish`,
    {
      method: "POST",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository security advisory publish failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositorySecurityAdvisoryDetail;
}

function appendRepositoryDependabotSearchParams(
  searchParams: URLSearchParams,
  query?: RepositoryDependabotAlertsQuery,
) {
  if (!query) {
    return;
  }
  const entries: [string, string | null | undefined][] = [
    ["state", query.state],
    ["q", query.query],
    ["package", query.package],
    ["ecosystem", query.ecosystem],
    ["manifest", query.manifest],
    ["scope", query.scope],
    ["severity", query.severity],
    ["sort", query.sort],
  ];
  for (const [key, value] of entries) {
    if (value?.trim()) {
      searchParams.set(key, value);
    }
  }
}

export async function getRepositoryDependabotAlertsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query?: RepositoryDependabotAlertsQuery,
): Promise<RepositoryDependabotAlertsFetchResult> {
  const searchParams = new URLSearchParams();
  appendRepositoryDependabotSearchParams(searchParams, query);
  const suffix = searchParams.size ? `?${searchParams.toString()}` : "";

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot${suffix}`,
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
      message: "Dependabot alerts are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Dependabot alerts are unavailable right now.";
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
    dependabot: (await response.json()) as RepositoryDependabotAlertsView,
  };
}

export async function getRepositoryDependabotAlertDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryDependabotAlertDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot/${encodeURIComponent(String(alertId))}`,
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
      message: "Dependabot alert detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Dependabot alert detail is unavailable right now.";
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
    dependabotAlert: (await response.json()) as RepositoryDependabotAlertDetail,
  };
}

function appendRepositoryCodeScanningSearchParams(
  searchParams: URLSearchParams,
  query?: RepositoryCodeScanningAlertsQuery,
) {
  if (!query) {
    return;
  }
  const entries: [string, string | null | undefined][] = [
    ["state", query.state],
    ["q", query.query],
    ["severity", query.severity],
    ["security_severity", query.securitySeverity],
    ["tool", query.tool],
    ["branch", query.branch],
    ["ref", query.ref],
    ["tag", query.tag],
    ["application_code", query.applicationCode],
    ["sort", query.sort],
  ];
  for (const [key, value] of entries) {
    if (value?.trim()) {
      searchParams.set(key, value);
    }
  }
}

function appendRepositorySecretScanningSearchParams(
  searchParams: URLSearchParams,
  query?: RepositorySecretScanningAlertsQuery,
) {
  if (!query) {
    return;
  }
  const entries: [string, string | null | undefined][] = [
    ["state", query.state],
    ["q", query.query],
    ["provider", query.provider],
    ["secret_type", query.secretType],
    ["validity", query.validity],
    ["resolution", query.resolution],
    ["bypassed", query.bypassed],
    ["team", query.team],
    ["topic", query.topic],
    ["sort", query.sort],
  ];
  for (const [key, value] of entries) {
    if (value?.trim()) {
      searchParams.set(key, value);
    }
  }
}

export async function getRepositoryCodeScanningAlertsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query?: RepositoryCodeScanningAlertsQuery,
): Promise<RepositoryCodeScanningAlertsFetchResult> {
  const searchParams = new URLSearchParams();
  appendRepositoryCodeScanningSearchParams(searchParams, query);
  const suffix = searchParams.size ? `?${searchParams.toString()}` : "";

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/code-scanning${suffix}`,
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
      message: "Code scanning alerts are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Code scanning alerts are unavailable right now.";
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
    codeScanning: (await response.json()) as RepositoryCodeScanningAlertsView,
  };
}

export async function getRepositoryCodeScanningAlertDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryCodeScanningAlertDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/code-scanning/${encodeURIComponent(String(alertId))}`,
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
      message: "Code scanning alert detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Code scanning alert detail is unavailable right now.";
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
    codeScanningAlert:
      (await response.json()) as RepositoryCodeScanningAlertDetail,
  };
}

export async function getRepositorySecretScanningAlertsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query?: RepositorySecretScanningAlertsQuery,
): Promise<RepositorySecretScanningAlertsFetchResult> {
  const searchParams = new URLSearchParams();
  appendRepositorySecretScanningSearchParams(searchParams, query);
  const suffix = searchParams.size ? `?${searchParams.toString()}` : "";

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/secret-scanning${suffix}`,
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
      message: "Secret scanning alerts are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Secret scanning alerts are unavailable right now.";
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
    secretScanning:
      (await response.json()) as RepositorySecretScanningAlertsView,
  };
}

export async function getRepositorySecretScanningAlertDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositorySecretScanningAlertDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/secret-scanning/${encodeURIComponent(String(alertId))}`,
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
      message: "Secret scanning alert detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Secret scanning alert detail is unavailable right now.";
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
    secretScanningAlert:
      (await response.json()) as RepositorySecretScanningAlertDetail,
  };
}

export async function mutateRepositorySecretScanningAlertFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
  mutation: RepositorySecretScanningAlertMutation,
): Promise<RepositorySecretScanningAlertDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/secret-scanning/${encodeURIComponent(String(alertId))}`,
    {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Secret scanning alert update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositorySecretScanningAlertDetail;
}

export async function mutateRepositoryCodeScanningAlertFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
  mutation: RepositoryCodeScanningAlertMutation,
): Promise<RepositoryCodeScanningAlertDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/code-scanning/${encodeURIComponent(String(alertId))}`,
    {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Code scanning alert update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryCodeScanningAlertDetail;
}

export async function createRepositoryCodeScanningIssueFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryCodeScanningAlertDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/code-scanning/${encodeURIComponent(String(alertId))}/issue`,
    {
      method: "POST",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Code scanning issue link failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryCodeScanningAlertDetail;
}

export async function mutateRepositoryDependabotAlertFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
  mutation: RepositoryDependabotAlertMutation,
): Promise<RepositoryDependabotAlertDetail> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot/${encodeURIComponent(String(alertId))}`,
    {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Dependabot alert update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryDependabotAlertDetail;
}

export async function bulkMutateRepositoryDependabotAlertsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryDependabotBulkMutation,
): Promise<RepositoryDependabotBulkMutationResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot/bulk`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify(mutation),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Dependabot bulk update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryDependabotBulkMutationResult;
}

export async function createRepositoryDependabotSecurityUpdateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  alertId: string | number,
): Promise<RepositoryDependabotSecurityUpdateResult> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot/${encodeURIComponent(String(alertId))}/security-update`,
    {
      method: "POST",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const envelope = ((await response.json().catch(() => null)) ??
      null) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Dependabot security update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryDependabotSecurityUpdateResult;
}

export async function getRepositoryNetworkFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryNetworkFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network`,
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
      message: "Repository Network is unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository Network is unavailable right now.";
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
    network: (await response.json()) as RepositoryNetworkView,
  };
}

function repositoryForksQueryString(options: RepositoryForksQuery = {}) {
  const params = new URLSearchParams();
  if (options.period?.trim()) params.set("period", options.period.trim());
  if (options.repositoryType?.trim()) {
    params.set("type", options.repositoryType.trim());
  }
  if (options.sort?.trim()) params.set("sort", options.sort.trim());
  const query = params.toString();
  return query ? `?${query}` : "";
}

export async function getRepositoryForksFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: RepositoryForksQuery = {},
): Promise<RepositoryForksFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/forks${repositoryForksQueryString(options)}`,
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
      message: "Repository forks are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository forks are unavailable right now.";
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
    forks: (await response.json()) as RepositoryForksView,
  };
}

function repositoryDependenciesQueryString(
  options: RepositoryDependenciesQuery = {},
) {
  const params = new URLSearchParams();
  if (options.query?.trim()) params.set("q", options.query.trim());
  if (options.ecosystem?.trim())
    params.set("ecosystem", options.ecosystem.trim());
  if (options.relationship?.trim()) {
    params.set("relationship", options.relationship.trim());
  }
  const query = params.toString();
  return query ? `?${query}` : "";
}

function repositoryDependentsQueryString(
  options: RepositoryDependentsQuery = {},
) {
  const params = new URLSearchParams();
  if (options.package?.trim()) params.set("package", options.package.trim());
  if (options.owner?.trim()) params.set("owner", options.owner.trim());
  const query = params.toString();
  return query ? `?${query}` : "";
}

export async function getRepositoryDependenciesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: RepositoryDependenciesQuery = {},
): Promise<RepositoryDependenciesFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network/dependencies${repositoryDependenciesQueryString(options)}`,
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
      message: "Repository dependencies are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository dependencies are unavailable right now.";
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
    dependencies: (await response.json()) as RepositoryDependenciesView,
  };
}

export async function getRepositoryDependentsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: RepositoryDependentsQuery = {},
): Promise<RepositoryDependentsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network/dependents${repositoryDependentsQueryString(options)}`,
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
      message: "Repository dependents are unavailable right now.",
    };
  }

  if (!response.ok) {
    let code: string | null = null;
    let message = "Repository dependents are unavailable right now.";
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
    dependents: (await response.json()) as RepositoryDependentsView,
  };
}

export async function startRepositorySbomExportFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositorySbomExport> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network/dependencies/sbom`,
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
    throw new Error(body?.error.message ?? "SBOM export failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositorySbomExport;
}

export async function downloadRepositorySbomExportFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  exportId: string,
): Promise<Response> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network/dependencies/sbom/${encodeURIComponent(exportId)}`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok && response.status !== 202) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "SBOM download failed", {
      cause: body,
    });
  }

  return response;
}

export async function saveRepositoryForkDefaultsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  options: Required<RepositoryForksQuery>,
): Promise<RepositoryForksView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/forks/defaults`,
    {
      method: "PUT",
      headers: {
        ...(cookie ? { cookie } : {}),
        "content-type": "application/json",
      },
      body: JSON.stringify(options),
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository fork defaults failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositoryForksView;
}

function repositorySettingsErrorResult(
  response: Response,
  fallback: string,
): Promise<{
  ok: false;
  status: number;
  code: string | null;
  message: string;
}> {
  return response
    .json()
    .then((body: { error?: { code?: string; message?: string } }) => ({
      ok: false as const,
      status: response.status,
      code: body.error?.code ?? null,
      message: body.error?.message ?? fallback,
    }))
    .catch(() => ({
      ok: false as const,
      status: response.status,
      code: null,
      message: fallback,
    }));
}

export async function getRepositoryWebhookSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryWebhookSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/hooks`,
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
      message: "Repository webhook settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository webhook settings are unavailable right now.",
    );
  }

  return {
    ok: true,
    settings: (await response.json()) as RepositoryWebhookSettings,
  };
}

export async function getOrganizationWebhookSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
): Promise<OrganizationWebhookSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/hooks`,
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
      message: "Organization webhook settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Organization webhook settings are unavailable right now.",
    );
  }

  return {
    ok: true,
    settings: (await response.json()) as OrganizationWebhookSettings,
  };
}

export async function getRepositoryActionsSecretsSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryActionsSecretsSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/secrets`,
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
      message: "Repository Actions secrets settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository Actions secrets settings are unavailable right now.",
    );
  }

  return {
    ok: true,
    settings: (await response.json()) as RepositoryActionsSecretsSettings,
  };
}

export async function getRepositoryActionsRunnerSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryActionsRunnerSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/actions/runners`,
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
      message: "Repository Actions runners are unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository Actions runners are unavailable right now.",
    );
  }

  return {
    ok: true,
    settings: (await response.json()) as RepositoryActionsRunnerSettings,
  };
}

export async function mutateRepositoryActionsRunnerSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryActionsRunnerMutation,
): Promise<
  | RepositoryActionsRunnerSettings
  | { assigned: ActionsRunnerJob[]; queuedJobs: number }
> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/actions/runners`;
  const request =
    mutation.action === "create-runner"
      ? {
          body: JSON.stringify({
            name: mutation.name,
            labels: mutation.labels,
          }),
          method: "POST",
          url: base,
        }
      : mutation.action === "update-settings"
        ? {
            body: JSON.stringify({
              concurrencyLimit: mutation.concurrencyLimit,
              cancelInProgress: mutation.cancelInProgress,
              githubTokenPermission: mutation.githubTokenPermission,
              allowPullRequestApproval: mutation.allowPullRequestApproval,
            }),
            method: "PATCH",
            url: base,
          }
        : { body: "{}", method: "POST", url: `${base}/schedule` };
  const response = await fetch(request.url, {
    method: request.method,
    headers: {
      "content-type": "application/json",
      ...(cookie ? { cookie } : {}),
    },
    body: request.body,
    cache: "no-store",
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw new Error("Repository Actions runner update failed.", {
      cause: (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_actions_runners_failed",
          message: "Repository Actions runner update failed.",
        },
        status: response.status,
      },
    });
  }
  return body as
    | RepositoryActionsRunnerSettings
    | { assigned: ActionsRunnerJob[]; queuedJobs: number };
}

export async function getRepositoryPagesSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryPagesSettingsFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/pages`,
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
      message: "Repository Pages settings are unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository Pages settings are unavailable right now.",
    );
  }

  return {
    ok: true,
    settings: (await response.json()) as RepositoryPagesSettings,
  };
}

export async function mutateRepositoryPagesSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryPagesMutation,
): Promise<RepositoryPagesSettings> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/pages`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "update-source": {
      const { action: _action, ...payload } = mutation;
      path = `${base}/source`;
      method = "PATCH";
      body = payload;
      break;
    }
    case "save-domain":
      path = `${base}/domain`;
      body = { domain: mutation.domain };
      break;
    case "remove-domain":
      path = `${base}/domain`;
      method = "DELETE";
      break;
    case "recheck-dns":
      path = `${base}/domain/recheck`;
      break;
    case "update-https":
      path = `${base}/https`;
      method = "PATCH";
      body = { enforced: mutation.enforced };
      break;
    case "request-deployment":
      path = `${base}/deployments`;
      break;
    case "unpublish-pages":
      path = `${base}/unpublish`;
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository Pages settings update failed.",
      { cause: envelope },
    );
  }

  const payload = (await response.json()) as
    | RepositoryPagesSettings
    | { settings: RepositoryPagesSettings };
  return "settings" in payload ? payload.settings : payload;
}

export async function mutateRepositoryActionsSecretsSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryActionsSecretsMutation,
): Promise<RepositoryActionsSecretsSettings> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/secrets`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "create-secret": {
      const { action: _action, ...payload } = mutation;
      path = `${base}/secrets`;
      body = payload;
      break;
    }
    case "update-secret": {
      const { action: _action, currentName, ...payload } = mutation;
      path = `${base}/secrets/${encodeURIComponent(currentName)}`;
      method = "PATCH";
      body = payload;
      break;
    }
    case "delete-secret":
      path = `${base}/secrets/${encodeURIComponent(mutation.name)}`;
      method = "DELETE";
      break;
    case "create-variable": {
      const { action: _action, ...payload } = mutation;
      path = `${base}/variables`;
      body = payload;
      break;
    }
    case "update-variable": {
      const { action: _action, currentName, ...payload } = mutation;
      path = `${base}/variables/${encodeURIComponent(currentName)}`;
      method = "PATCH";
      body = payload;
      break;
    }
    case "delete-variable":
      path = `${base}/variables/${encodeURIComponent(mutation.name)}`;
      method = "DELETE";
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository Actions secrets update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryActionsSecretsSettings;
}

export async function getRepositoryWebhookDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  hookId: string,
): Promise<RepositoryWebhookDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/hooks/${encodeURIComponent(hookId)}`,
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
      message: "Repository webhook detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository webhook detail is unavailable right now.",
    );
  }

  return {
    ok: true,
    detail: (await response.json()) as RepositoryWebhookDetail,
  };
}

export async function getOrganizationWebhookDetailFromCookie(
  cookie: string | null | undefined,
  org: string,
  hookId: string,
): Promise<RepositoryWebhookDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/hooks/${encodeURIComponent(hookId)}`,
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
      message: "Organization webhook detail is unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Organization webhook detail is unavailable right now.",
    );
  }

  return {
    ok: true,
    detail: (await response.json()) as RepositoryWebhookDetail,
  };
}

export async function getRepositoryWebhookDeliveryDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  hookId: string,
  deliveryId: string,
): Promise<WebhookDeliveryDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/hooks/${encodeURIComponent(hookId)}/deliveries/${encodeURIComponent(deliveryId)}`,
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
      message: "Repository webhook delivery is unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Repository webhook delivery is unavailable right now.",
    );
  }

  return {
    ok: true,
    delivery: (await response.json()) as WebhookDeliveryDetail,
  };
}

export async function getOrganizationWebhookDeliveryDetailFromCookie(
  cookie: string | null | undefined,
  org: string,
  hookId: string,
  deliveryId: string,
): Promise<WebhookDeliveryDetailFetchResult> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/hooks/${encodeURIComponent(hookId)}/deliveries/${encodeURIComponent(deliveryId)}`,
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
      message: "Organization webhook delivery is unavailable right now.",
    };
  }

  if (!response.ok) {
    return repositorySettingsErrorResult(
      response,
      "Organization webhook delivery is unavailable right now.",
    );
  }

  return {
    ok: true,
    delivery: (await response.json()) as WebhookDeliveryDetail,
  };
}

export async function mutateRepositoryWebhookSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryWebhookMutation,
): Promise<RepositoryWebhookMutationResult> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/hooks`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "create-webhook": {
      const { action: _action, ...payload } = mutation;
      body = payload;
      break;
    }
    case "update-webhook": {
      const { action: _action, hookId, ...payload } = mutation;
      path = `${base}/${encodeURIComponent(hookId)}`;
      method = "PATCH";
      body = payload;
      break;
    }
    case "delete-webhook":
      path = `${base}/${encodeURIComponent(mutation.hookId)}`;
      method = "DELETE";
      break;
    case "ping-webhook":
      path = `${base}/${encodeURIComponent(mutation.hookId)}/ping`;
      break;
    case "redeliver-delivery":
      path = `${base}/${encodeURIComponent(mutation.hookId)}/deliveries/${encodeURIComponent(mutation.deliveryId)}/redeliver`;
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository webhook update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryWebhookMutationResult;
}

export async function mutateOrganizationWebhookSettingsFromCookie(
  cookie: string | null | undefined,
  org: string,
  mutation: RepositoryWebhookMutation,
): Promise<RepositoryWebhookMutationResult> {
  const base = `${apiBaseUrl()}/api/orgs/${encodeURIComponent(org)}/settings/hooks`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "create-webhook": {
      const { action: _action, ...payload } = mutation;
      body = payload;
      break;
    }
    case "update-webhook": {
      const { action: _action, hookId, ...payload } = mutation;
      path = `${base}/${encodeURIComponent(hookId)}`;
      method = "PATCH";
      body = payload;
      break;
    }
    case "delete-webhook":
      path = `${base}/${encodeURIComponent(mutation.hookId)}`;
      method = "DELETE";
      break;
    case "ping-webhook":
      path = `${base}/${encodeURIComponent(mutation.hookId)}/ping`;
      break;
    case "redeliver-delivery":
      path = `${base}/${encodeURIComponent(mutation.hookId)}/deliveries/${encodeURIComponent(mutation.deliveryId)}/redeliver`;
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Organization webhook update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryWebhookMutationResult;
}

export async function mutateRepositoryBranchSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryBranchPolicyMutation,
): Promise<RepositoryBranchSettings> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/branches`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "create-rule": {
      path = `${base}/rules`;
      const { action: _action, ruleId: _ruleId, ...payload } = mutation;
      body = payload;
      break;
    }
    case "update-rule": {
      path = `${base}/rules/${encodeURIComponent(mutation.ruleId ?? "")}`;
      method = "PATCH";
      const { action: _action, ruleId: _ruleId, ...payload } = mutation;
      body = payload;
      break;
    }
    case "delete-rule":
      path = `${base}/rules/${encodeURIComponent(mutation.ruleId)}`;
      method = "DELETE";
      break;
    case "create-ruleset": {
      path = `${base}/rulesets`;
      const { action: _action, rulesetId: _rulesetId, ...payload } = mutation;
      body = payload;
      break;
    }
    case "update-ruleset": {
      path = `${base}/rulesets/${encodeURIComponent(mutation.rulesetId ?? "")}`;
      method = "PATCH";
      const { action: _action, rulesetId: _rulesetId, ...payload } = mutation;
      body = payload;
      break;
    }
    case "delete-ruleset":
      path = `${base}/rulesets/${encodeURIComponent(mutation.rulesetId)}`;
      method = "DELETE";
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository branch policy update failed.",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryBranchSettings;
}

export type RepositoryAccessMutation =
  | {
      action: "invite-person";
      emailOrLogin: string;
      role: Exclude<RepositoryAccessRole, "owner">;
    }
  | {
      action: "grant-team";
      teamSlug: string;
      role: Exclude<RepositoryAccessRole, "owner">;
    }
  | {
      action: "update-person-role";
      userId: string;
      role: Exclude<RepositoryAccessRole, "owner">;
    }
  | {
      action: "update-team-role";
      teamId: string;
      role: Exclude<RepositoryAccessRole, "owner">;
    }
  | { action: "remove-person"; userId: string }
  | { action: "remove-team"; teamId: string }
  | { action: "cancel-invitation"; invitationId: string };

export async function mutateRepositoryAccessSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  mutation: RepositoryAccessMutation,
): Promise<RepositoryAccessSettings> {
  const base = `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/access`;
  let path = base;
  let method = "POST";
  let body: unknown;

  switch (mutation.action) {
    case "invite-person":
      body = { emailOrLogin: mutation.emailOrLogin, role: mutation.role };
      break;
    case "grant-team":
      path = `${base}/teams`;
      body = { teamSlug: mutation.teamSlug, role: mutation.role };
      break;
    case "update-person-role":
      path = `${base}/collaborators/${encodeURIComponent(mutation.userId)}`;
      method = "PATCH";
      body = { role: mutation.role };
      break;
    case "update-team-role":
      path = `${base}/teams/${encodeURIComponent(mutation.teamId)}`;
      method = "PATCH";
      body = { role: mutation.role };
      break;
    case "remove-person":
      path = `${base}/collaborators/${encodeURIComponent(mutation.userId)}`;
      method = "DELETE";
      break;
    case "remove-team":
      path = `${base}/teams/${encodeURIComponent(mutation.teamId)}`;
      method = "DELETE";
      break;
    case "cancel-invitation":
      path = `${base}/invitations/${encodeURIComponent(mutation.invitationId)}`;
      method = "DELETE";
      break;
  }

  const response = await fetch(path, {
    method,
    headers: {
      ...(body ? { "content-type": "application/json" } : {}),
      ...(cookie ? { cookie } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (!response.ok) {
    const envelope = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Repository access update failed",
      { cause: envelope },
    );
  }

  return (await response.json()) as RepositoryAccessSettings;
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
    const contentsPath = encodedPath ? `/contents/${encodedPath}` : "/contents";
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}${contentsPath}?${params.toString()}`,
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
  options: {
    author?: string | null;
    until?: string | null;
    page?: number | null;
    pageSize?: number | null;
  } = {},
): Promise<RepositoryCommitHistoryView | null> {
  const params = new URLSearchParams({ ref: refName });
  const normalizedPath = path.replace(/^\/+|\/+$/g, "");
  if (normalizedPath) {
    params.set("path", normalizedPath);
  }
  if (options.author?.trim()) {
    params.set("author", options.author.trim());
  }
  if (options.until?.trim()) {
    params.set("until", options.until.trim());
  }
  if (options.page && Number.isFinite(options.page)) {
    params.set("page", String(options.page));
  }
  if (options.pageSize && Number.isFinite(options.pageSize)) {
    params.set("pageSize", String(options.pageSize));
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

  return (await response.json()) as RepositoryCommitHistoryView;
}

export async function getRepositoryCommitDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  sha: string,
): Promise<RepositoryCommitDetailView | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/commits/${encodeURIComponent(sha)}`,
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

  return (await response.json()) as RepositoryCommitDetailView;
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

export async function getRepositoryStargazersFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: Pick<ProfileRepositoryListQuery, "page" | "pageSize"> = {},
): Promise<RepositoryStargazerList | null> {
  let response: Response;
  try {
    const url = new URL(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/stargazers`,
    );
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

  return (await response.json()) as RepositoryStargazerList;
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

export async function getRepositoryWatchSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<RepositoryWatchSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/watch`,
    {
      method: "GET",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  if (!response.ok) {
    const body = (await response
      .json()
      .catch(() => null)) as ApiErrorEnvelope | null;
    throw new Error(body?.error.message ?? "Repository watch settings failed", {
      cause: body,
    });
  }

  return (await response.json()) as RepositoryWatchSettings;
}

export async function updateRepositoryWatchSettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  patch: RepositoryWatchSettingsPatch,
): Promise<RepositoryWatchSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/watch`,
    {
      method: "PATCH",
      headers: {
        ...(cookie ? { cookie } : {}),
        "content-type": "application/json",
      },
      body: JSON.stringify(patch),
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

  return (await response.json()) as RepositoryWatchSettings;
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

export function organizationSlugAvailabilityPath(name: string): string {
  const params = new URLSearchParams({ name });
  return `/api/organizations/slug-availability?${params.toString()}`;
}

export async function getOrganizationSlugAvailabilityFromCookie(
  cookie: string | null | undefined,
  name: string,
): Promise<OrganizationSlugAvailability | null> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${organizationSlugAvailabilityPath(name)}`,
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

  return (await response.json()) as OrganizationSlugAvailability;
}

export async function createOrganizationFromCookie(
  cookie: string | null | undefined,
  request: CreateOrganizationRequest,
): Promise<CreatedOrganization> {
  const response = await fetch(`${apiBaseUrl()}/api/organizations`, {
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
      body?.error.message ?? "Organization could not be created",
      {
        cause: body,
      },
    );
  }

  return (await response.json()) as CreatedOrganization;
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

export type NotificationTriageAction =
  | "read"
  | "unread"
  | "save"
  | "unsave"
  | "done"
  | "inbox"
  | "subscribe"
  | "unsubscribe";

export type NotificationFolderCounts = {
  inbox: number;
  saved: number;
  done: number;
};

export type NotificationTriageResponse = {
  id: string;
  unread: boolean;
  saved: boolean;
  done: boolean;
  subscribed: boolean;
  lastReadAt: string | null;
  savedAt: string | null;
  unreadCount: number;
  folderCounts: NotificationFolderCounts;
};

export type NotificationBulkFailure = {
  id: string;
  code: string;
  message: string;
};

export type NotificationBulkTriageResponse = {
  action: NotificationTriageAction;
  updated: NotificationTriageResponse[];
  failed: NotificationBulkFailure[];
  unreadCount: number;
  folderCounts: NotificationFolderCounts;
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

export type NotificationDefaultFilter = {
  id: string;
  name: string;
  queryString: string;
  href: string;
};

export type NotificationCustomFilter = {
  id: string;
  name: string;
  queryString: string;
  position: number;
  href: string;
  createdAt: string;
  updatedAt: string;
};

export type NotificationFilterSettings = {
  defaultFilters: NotificationDefaultFilter[];
  customFilters: NotificationCustomFilter[];
  limit: number;
  remaining: number;
  allowedQualifiers: string[];
};

export type UpsertNotificationCustomFilterRequest = {
  name: string;
  queryString: string;
};

export type NotificationDeliveryEmail = {
  id: string;
  email: string;
  isPrimary: boolean;
  isPublic: boolean;
  verified: boolean;
};

export type NotificationDeliveryPreference = {
  key: string;
  label: string;
  section: "subscriptions" | "system" | string;
  description: string;
  channels: string[];
  supportedChannels: string[];
  disabled: boolean;
  disabledReason: string | null;
};

export type NotificationDeliverySettings = {
  defaultEmailId: string | null;
  defaultEmail: string | null;
  emailChannelAvailable: boolean;
  sesSenderReady: boolean;
  emails: NotificationDeliveryEmail[];
  preferences: NotificationDeliveryPreference[];
  customRoutingHref: string;
  watchedRepositoriesHref: string;
  ignoredRepositoriesHref: string;
};

export type UpdateNotificationDeliverySettingsRequest = {
  defaultEmailId?: string | null;
  preferences?: { key: string; channels: string[] }[];
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

export async function getNotificationFilterSettingsFromCookie(
  cookie: string | null | undefined,
): Promise<NotificationFilterSettings | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}/api/notifications/custom-filters`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Notification filters are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "notification_filters_failed",
          message: "Notification filters could not be loaded.",
        },
        status: response.status,
      }
    );
  }
  return body as NotificationFilterSettings;
}

export async function getNotificationDeliverySettingsFromCookie(
  cookie: string | null | undefined,
): Promise<NotificationDeliverySettings | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/notifications/delivery-preferences`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Notification delivery settings are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "notification_delivery_failed",
          message: "Notification delivery settings could not be loaded.",
        },
        status: response.status,
      }
    );
  }
  return body as NotificationDeliverySettings;
}

export async function updateNotificationDeliverySettingsFromCookie(
  cookie: string | null | undefined,
  input: UpdateNotificationDeliverySettingsRequest,
): Promise<NotificationDeliverySettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/notifications/delivery-preferences`,
    {
      method: "PATCH",
      headers: {
        ...(cookie ? { cookie } : {}),
        "content-type": "application/json",
      },
      body: JSON.stringify(input),
      cache: "no-store",
    },
  );
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw notificationDeliveryError(body, response.status);
  }
  return body as NotificationDeliverySettings;
}

export async function createNotificationCustomFilterFromCookie(
  cookie: string | null | undefined,
  input: UpsertNotificationCustomFilterRequest,
): Promise<NotificationFilterSettings> {
  return writeNotificationCustomFilter(
    cookie,
    "/api/notifications/custom-filters",
    "POST",
    input,
  );
}

export async function updateNotificationCustomFilterFromCookie(
  cookie: string | null | undefined,
  filterId: string,
  input: UpsertNotificationCustomFilterRequest,
): Promise<NotificationFilterSettings> {
  return writeNotificationCustomFilter(
    cookie,
    `/api/notifications/custom-filters/${encodeURIComponent(filterId)}`,
    "PATCH",
    input,
  );
}

export async function deleteNotificationCustomFilterFromCookie(
  cookie: string | null | undefined,
  filterId: string,
): Promise<NotificationFilterSettings> {
  const response = await fetch(
    `${apiBaseUrl()}/api/notifications/custom-filters/${encodeURIComponent(filterId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw notificationFilterError(body, response.status);
  }
  return body as NotificationFilterSettings;
}

async function writeNotificationCustomFilter(
  cookie: string | null | undefined,
  path: string,
  method: "POST" | "PATCH",
  input: UpsertNotificationCustomFilterRequest,
): Promise<NotificationFilterSettings> {
  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method,
    headers: {
      ...(cookie ? { cookie } : {}),
      "content-type": "application/json",
    },
    body: JSON.stringify(input),
    cache: "no-store",
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw notificationFilterError(body, response.status);
  }
  return body as NotificationFilterSettings;
}

function notificationFilterError(body: unknown, status: number) {
  const fallback: ApiErrorEnvelope = {
    error: {
      code: "notification_filters_failed",
      message: "Notification filters could not be saved.",
    },
    status,
  };
  return new Error("Notification filter operation failed", {
    cause: (body as ApiErrorEnvelope | null) ?? fallback,
  });
}

function notificationDeliveryError(body: unknown, status: number) {
  const fallback: ApiErrorEnvelope = {
    error: {
      code: "notification_delivery_failed",
      message: "Notification delivery settings could not be saved.",
    },
    status,
  };
  return new Error("Notification delivery operation failed", {
    cause: (body as ApiErrorEnvelope | null) ?? fallback,
  });
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

export async function updateNotificationTriageFromCookie(
  cookie: string | null | undefined,
  notificationId: string,
  action: NotificationTriageAction,
): Promise<NotificationTriageResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/notifications/${encodeURIComponent(notificationId)}/${action}`,
    {
      method: "PATCH",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    const fallback: ApiErrorEnvelope = {
      error: {
        code: "notification_triage_failed",
        message: "Notification could not be updated.",
      },
      status: response.status,
    };
    throw new Error("Notification triage failed", {
      cause: (body as ApiErrorEnvelope | null) ?? fallback,
    });
  }
  return body as NotificationTriageResponse;
}

export async function bulkUpdateNotificationTriageFromCookie(
  cookie: string | null | undefined,
  notificationIds: string[],
  action: NotificationTriageAction,
): Promise<NotificationBulkTriageResponse> {
  const response = await fetch(`${apiBaseUrl()}/api/notifications/bulk`, {
    method: "POST",
    headers: {
      ...(cookie ? { cookie } : {}),
      "content-type": "application/json",
    },
    body: JSON.stringify({ notificationIds, action }),
    cache: "no-store",
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    const fallback: ApiErrorEnvelope = {
      error: {
        code: "notification_bulk_triage_failed",
        message: "Notifications could not be updated.",
      },
      status: response.status,
    };
    throw new Error("Notification bulk triage failed", {
      cause: (body as ApiErrorEnvelope | null) ?? fallback,
    });
  }
  return body as NotificationBulkTriageResponse;
}

export async function getRepositoryDiscussionsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryDiscussionsQuery = {},
  categorySlug?: string | null,
): Promise<RepositoryDiscussionsView | ApiErrorEnvelope> {
  const path = repositoryDiscussionsPath(owner, repo, query, categorySlug);
  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl()}${path}`, {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    });
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Repository discussions are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_discussions_failed",
          message: "Repository discussions could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryDiscussionsView;
}

export async function getRepositoryDiscussionCreationFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  query: RepositoryDiscussionCreationQuery = {},
): Promise<DiscussionCreationView | ApiErrorEnvelope> {
  const params = new URLSearchParams();
  if (query.category) params.set("category", query.category);
  if (query.title) params.set("title", query.title);
  const search = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/new${search ? `?${search}` : ""}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Discussion creation is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_discussion_creation_failed",
          message: "Discussion creation metadata could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as DiscussionCreationView;
}

export async function getRepositoryDiscussionDetailFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  query: RepositoryDiscussionDetailQuery = {},
): Promise<RepositoryDiscussionDetailView | ApiErrorEnvelope> {
  const params = new URLSearchParams();
  if (query.sort) params.set("sort", query.sort);
  if (query.page) params.set("page", String(query.page));
  if (query.pageSize) params.set("page_size", String(query.pageSize));
  const search = params.toString();
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}${search ? `?${search}` : ""}`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Discussion detail is temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_discussion_detail_failed",
          message: "Discussion detail could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as RepositoryDiscussionDetailView;
}

export async function getRepositoryDiscussionCategorySettingsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
): Promise<DiscussionCategorySettingsView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Discussion category settings are temporarily unavailable.",
      },
      status: 503,
    };
  }

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (body as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_discussion_category_settings_failed",
          message: "Discussion category settings could not be loaded.",
        },
        status: response.status,
      }
    );
  }

  return body as DiscussionCategorySettingsView;
}

export async function createRepositoryDiscussionCategoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: CreateDiscussionCategoryRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories`,
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
      envelope?.error.message ?? "Discussion category could not be created.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function updateRepositoryDiscussionCategoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  categoryId: string,
  request: UpdateDiscussionCategoryRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}`,
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
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion category could not be updated.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function createRepositoryDiscussionCategorySectionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: CreateDiscussionCategorySectionRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections`,
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
      envelope?.error.message ??
        "Discussion category section could not be created.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function updateRepositoryDiscussionCategorySectionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  sectionId: string,
  request: UpdateDiscussionCategorySectionRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections/${encodeURIComponent(sectionId)}`,
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
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion category section could not be updated.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function deleteRepositoryDiscussionCategorySectionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  sectionId: string,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections/${encodeURIComponent(sectionId)}`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion category section could not be deleted.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function reorderRepositoryDiscussionCategoriesFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: DiscussionCategoryOrderRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/order`,
    {
      method: "PUT",
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
      envelope?.error.message ??
        "Discussion category order could not be saved.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function reorderRepositoryDiscussionSectionsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: DiscussionSectionOrderRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/sections/order`,
    {
      method: "PUT",
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
      envelope?.error.message ??
        "Discussion category section order could not be saved.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function deleteRepositoryDiscussionCategoryFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  categoryId: string,
  request: DeleteDiscussionCategoryRequest,
): Promise<DiscussionCategorySettingsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}`,
    {
      method: "DELETE",
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
      envelope?.error.message ?? "Discussion category could not be deleted.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategorySettingsView;
}

export async function getRepositoryDiscussionCategoryTemplateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  categoryId: string,
): Promise<DiscussionCategoryTemplateView | ApiErrorEnvelope> {
  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}/template`,
      {
        headers: cookie ? { cookie } : undefined,
        cache: "no-store",
      },
    );
  } catch {
    return {
      error: {
        code: "network_error",
        message: "Discussion category template is temporarily unavailable.",
      },
      status: 503,
    };
  }
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    return (
      (payload as ApiErrorEnvelope | null) ?? {
        error: {
          code: "repository_discussion_category_template_failed",
          message: "Discussion category template could not be loaded.",
        },
        status: response.status,
      }
    );
  }
  return payload as DiscussionCategoryTemplateView;
}

export async function previewRepositoryDiscussionCategoryTemplateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  categoryId: string,
  request: DiscussionCategoryTemplatePreviewRequest,
): Promise<DiscussionFormDefinition> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}/template/preview`,
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
      envelope?.error.message ??
        "Discussion category template preview could not be generated.",
      { cause: envelope },
    );
  }
  return payload as DiscussionFormDefinition;
}

export async function commitRepositoryDiscussionCategoryTemplateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  categoryId: string,
  request: DiscussionCategoryTemplateCommitRequest,
): Promise<DiscussionCategoryTemplateCommitResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}/template`,
    {
      method: "PUT",
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
      envelope?.error.message ??
        "Discussion category template could not be committed.",
      { cause: envelope },
    );
  }
  return payload as DiscussionCategoryTemplateCommitResponse;
}

export async function createRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  request: CreateDiscussionRequest,
): Promise<CreateDiscussionResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions`,
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
      envelope?.error.message ?? "Discussion could not be created.",
      { cause: envelope },
    );
  }

  return payload as CreateDiscussionResponse;
}

export async function setRepositoryDiscussionVoteFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  voted: boolean,
): Promise<DiscussionVoteResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/vote`,
    {
      method: voted ? "PUT" : "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion vote could not be updated.",
      { cause: envelope },
    );
  }

  return payload as DiscussionVoteResponse;
}

export async function voteRepositoryDiscussionPollFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: DiscussionPollVoteRequest,
): Promise<DiscussionPollVoteResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/poll/vote`,
    {
      method: "PUT",
      headers: {
        ...(cookie ? { cookie } : {}),
        "content-type": "application/json",
      },
      body: JSON.stringify(request),
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion poll vote could not be updated.",
      { cause: envelope },
    );
  }

  return payload as DiscussionPollVoteResponse;
}

export async function createRepositoryDiscussionCommentFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: CreateDiscussionCommentRequest,
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/comments`,
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
      envelope?.error.message ?? "Discussion comment could not be created.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function createRepositoryDiscussionReplyFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  commentId: string,
  request: CreateDiscussionCommentRequest,
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/comments/${encodeURIComponent(commentId)}/replies`,
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
      envelope?.error.message ?? "Discussion reply could not be created.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function setRepositoryDiscussionReactionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  content: DiscussionReactionContent,
  reacted: boolean,
  commentId?: string,
): Promise<DiscussionReactionSummary[]> {
  const commentSegment = commentId
    ? `/comments/${encodeURIComponent(commentId)}`
    : "";
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}${commentSegment}/reactions`,
    {
      method: reacted ? "PUT" : "DELETE",
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
      envelope?.error.message ?? "Discussion reaction could not be updated.",
      { cause: envelope },
    );
  }

  return payload as DiscussionReactionSummary[];
}

export async function setRepositoryDiscussionSubscriptionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  subscribed: boolean,
): Promise<DiscussionSubscriptionState> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/subscription`,
    {
      method: subscribed ? "PUT" : "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion notification subscription could not be updated.",
      { cause: envelope },
    );
  }

  return payload as DiscussionSubscriptionState;
}

export async function setRepositoryDiscussionAnswerFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  commentId: string,
  marked: boolean,
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/answer`,
    {
      method: marked ? "PUT" : "DELETE",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ commentId }),
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion answer state could not be updated.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function updateRepositoryDiscussionStateFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  state: "open" | "closed",
  reason?: string,
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/state`,
    {
      method: "PUT",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({ state, reason }),
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion state could not be updated.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function pinRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: {
    target: "global" | "category";
    categorySlug?: string;
    title?: string;
    body?: string;
  },
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/pin`,
    {
      method: "PUT",
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
    throw new Error(envelope?.error.message ?? "Discussion pin failed.", {
      cause: envelope,
    });
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function updateRepositoryDiscussionPinFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: { title?: string; body?: string },
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/pin`,
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

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion pin update failed.",
      {
        cause: envelope,
      },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function unpinRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/pin`,
    {
      method: "DELETE",
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(envelope?.error.message ?? "Discussion unpin failed.", {
      cause: envelope,
    });
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function setRepositoryDiscussionLockFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  locked: boolean,
  request: { allowReactions?: boolean } = {},
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/lock`,
    {
      method: locked ? "PUT" : "DELETE",
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
    throw new Error(envelope?.error.message ?? "Discussion lock failed.", {
      cause: envelope,
    });
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function recategorizeRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: { categorySlug: string },
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/category`,
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

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion category could not be changed.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function updateRepositoryDiscussionMetadataFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: { categorySlug?: string; labelIds?: string[] },
): Promise<RepositoryDiscussionDetailView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/metadata`,
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

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ?? "Discussion metadata could not be updated.",
      { cause: envelope },
    );
  }

  return payload as RepositoryDiscussionDetailView;
}

export async function getRepositoryDiscussionTransferTargetsFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
): Promise<DiscussionTransferTargetsView> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/transfer-targets`,
    {
      headers: cookie ? { cookie } : undefined,
      cache: "no-store",
    },
  );
  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const envelope = payload as ApiErrorEnvelope | null;
    throw new Error(
      envelope?.error.message ??
        "Discussion transfer targets could not be loaded.",
      { cause: envelope },
    );
  }
  return payload as DiscussionTransferTargetsView;
}

export async function transferRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: { repositoryId: string; categorySlug: string },
): Promise<TransferDiscussionResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/transfer`,
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
    throw new Error(envelope?.error.message ?? "Discussion transfer failed.", {
      cause: envelope,
    });
  }
  return payload as TransferDiscussionResponse;
}

export async function deleteRepositoryDiscussionFromCookie(
  cookie: string | null | undefined,
  owner: string,
  repo: string,
  discussionNumber: number | string,
  request: { confirmation: string; reason?: string },
): Promise<DeleteDiscussionResponse> {
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/delete`,
    {
      method: "DELETE",
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
    throw new Error(envelope?.error.message ?? "Discussion delete failed.", {
      cause: envelope,
    });
  }
  return payload as DeleteDiscussionResponse;
}

function repositoryDiscussionsPath(
  owner: string,
  repo: string,
  query: RepositoryDiscussionsQuery,
  categorySlug?: string | null,
): string {
  const base = categorySlug
    ? `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/categories/${encodeURIComponent(categorySlug)}`
    : `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions`;
  const params = new URLSearchParams();
  if (query.q) params.set("q", query.q);
  if (query.label) params.set("label", query.label);
  if (query.state) params.set("state", query.state);
  if (query.answered !== undefined)
    params.set("answered", String(query.answered));
  if (query.locked !== undefined) params.set("locked", String(query.locked));
  if (query.pinned !== undefined) params.set("pinned", String(query.pinned));
  if (query.sort) params.set("sort", query.sort);
  if (query.page) params.set("page", String(query.page));
  if (query.pageSize) params.set("page_size", String(query.pageSize));
  const search = params.toString();
  return search ? `${base}?${search}` : base;
}
