CREATE TABLE pages_sites (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    source_kind text NOT NULL DEFAULT 'none',
    source_branch text,
    source_folder text,
    workflow_id uuid REFERENCES actions_workflows(id) ON DELETE SET NULL,
    workflow_artifact_name text,
    default_site_url text NOT NULL,
    custom_domain text,
    dns_challenge_name text,
    dns_challenge_value text,
    dns_status text NOT NULL DEFAULT 'not_configured',
    https_enforced boolean NOT NULL DEFAULT false,
    certificate_status text NOT NULL DEFAULT 'not_configured',
    provisioning_status text NOT NULL DEFAULT 'not_configured',
    cloudfront_distribution_id text,
    cloudfront_alias text,
    s3_artifact_prefix text,
    last_deployment_id uuid,
    unpublished_at timestamptz,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pages_sites_source_kind_check CHECK (source_kind IN ('none', 'branch', 'actions')),
    CONSTRAINT pages_sites_source_branch_required CHECK (
        source_kind <> 'branch'
        OR (length(trim(COALESCE(source_branch, ''))) > 0 AND source_folder IN ('/', '/docs'))
    ),
    CONSTRAINT pages_sites_actions_source_required CHECK (
        source_kind <> 'actions'
        OR workflow_id IS NOT NULL
    ),
    CONSTRAINT pages_sites_dns_status_check CHECK (dns_status IN ('not_configured', 'pending', 'verified', 'misconfigured')),
    CONSTRAINT pages_sites_certificate_status_check CHECK (certificate_status IN ('not_configured', 'pending', 'issued', 'failed')),
    CONSTRAINT pages_sites_provisioning_status_check CHECK (provisioning_status IN ('not_configured', 'queued', 'provisioning', 'ready', 'failed', 'unpublished')),
    CONSTRAINT pages_sites_custom_domain_not_blank CHECK (custom_domain IS NULL OR length(trim(custom_domain)) > 0),
    CONSTRAINT pages_sites_https_requires_domain CHECK (https_enforced = false OR custom_domain IS NOT NULL)
);

CREATE UNIQUE INDEX pages_sites_repository_unique
ON pages_sites (repository_id);

CREATE UNIQUE INDEX pages_sites_active_custom_domain_unique
ON pages_sites (lower(custom_domain))
WHERE custom_domain IS NOT NULL AND unpublished_at IS NULL;

CREATE INDEX pages_sites_repository_status_idx
ON pages_sites (repository_id, source_kind, provisioning_status);

CREATE TRIGGER pages_sites_set_updated_at
BEFORE UPDATE ON pages_sites
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE pages_deployments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    site_id uuid NOT NULL REFERENCES pages_sites(id) ON DELETE CASCADE,
    source_kind text NOT NULL,
    source_branch text,
    source_folder text,
    commit_id uuid REFERENCES commits(id) ON DELETE SET NULL,
    workflow_run_id uuid REFERENCES workflow_runs(id) ON DELETE SET NULL,
    workflow_artifact_id uuid REFERENCES workflow_artifacts(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'queued',
    conclusion text,
    default_url text NOT NULL,
    custom_domain_url text,
    artifact_storage_key text,
    artifact_manifest jsonb NOT NULL DEFAULT '{}'::jsonb,
    build_log_excerpt text,
    failure_reason text,
    requested_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    queued_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    completed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pages_deployments_source_kind_check CHECK (source_kind IN ('branch', 'actions')),
    CONSTRAINT pages_deployments_status_check CHECK (status IN ('queued', 'building', 'deployed', 'failed', 'cancelled', 'unpublished')),
    CONSTRAINT pages_deployments_conclusion_check CHECK (
        conclusion IS NULL OR conclusion IN ('success', 'failure', 'cancelled', 'skipped')
    ),
    CONSTRAINT pages_deployments_artifact_manifest_object CHECK (jsonb_typeof(artifact_manifest) = 'object')
);

CREATE INDEX pages_deployments_repository_created_idx
ON pages_deployments (repository_id, created_at DESC);

CREATE INDEX pages_deployments_site_status_idx
ON pages_deployments (site_id, status, created_at DESC);

CREATE TRIGGER pages_deployments_set_updated_at
BEFORE UPDATE ON pages_deployments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE pages_sites
ADD CONSTRAINT pages_sites_last_deployment_fk
FOREIGN KEY (last_deployment_id) REFERENCES pages_deployments(id) ON DELETE SET NULL;

CREATE TABLE pages_domain_verifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id uuid NOT NULL REFERENCES pages_sites(id) ON DELETE CASCADE,
    domain text NOT NULL,
    challenge_name text NOT NULL,
    challenge_value text NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    checked_at timestamptz,
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pages_domain_verifications_status_check CHECK (status IN ('pending', 'verified', 'misconfigured'))
);

CREATE INDEX pages_domain_verifications_site_created_idx
ON pages_domain_verifications (site_id, created_at DESC);

CREATE TRIGGER pages_domain_verifications_set_updated_at
BEFORE UPDATE ON pages_domain_verifications
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE pages_build_artifacts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_id uuid NOT NULL REFERENCES pages_deployments(id) ON DELETE CASCADE,
    path text NOT NULL,
    storage_key text NOT NULL,
    content_type text,
    byte_size bigint NOT NULL DEFAULT 0,
    checksum text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pages_build_artifacts_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT pages_build_artifacts_storage_key_not_blank CHECK (length(trim(storage_key)) > 0),
    CONSTRAINT pages_build_artifacts_byte_size_non_negative CHECK (byte_size >= 0)
);

CREATE UNIQUE INDEX pages_build_artifacts_deployment_path_unique
ON pages_build_artifacts (deployment_id, lower(path));
