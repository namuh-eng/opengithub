CREATE TABLE actions_secrets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    scope_kind text NOT NULL DEFAULT 'repository',
    scope_name text,
    name text NOT NULL,
    encrypted_value_ciphertext text NOT NULL,
    encrypted_value_nonce text NOT NULL,
    storage_kind text NOT NULL DEFAULT 'local_envelope',
    value_fingerprint text NOT NULL,
    visibility_policy text NOT NULL DEFAULT 'private',
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT actions_secrets_scope_kind_check CHECK (scope_kind IN ('repository', 'organization', 'environment')),
    CONSTRAINT actions_secrets_storage_kind_check CHECK (storage_kind IN ('local_envelope', 'aws_kms', 'aws_secrets_manager', 's3_envelope')),
    CONSTRAINT actions_secrets_visibility_policy_check CHECK (visibility_policy IN ('private', 'selected_repositories', 'all_repositories')),
    CONSTRAINT actions_secrets_name_identifier CHECK (name ~ '^[A-Z_][A-Z0-9_]*$'),
    CONSTRAINT actions_secrets_scope_name_required CHECK (scope_kind = 'repository' OR length(trim(COALESCE(scope_name, ''))) > 0)
);

CREATE UNIQUE INDEX actions_secrets_repository_scope_name_unique
ON actions_secrets (repository_id, scope_kind, COALESCE(scope_name, ''), name);

CREATE INDEX actions_secrets_repository_updated_idx
ON actions_secrets (repository_id, updated_at DESC);

CREATE TRIGGER actions_secrets_set_updated_at
BEFORE UPDATE ON actions_secrets
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE actions_variables (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    scope_kind text NOT NULL DEFAULT 'repository',
    scope_name text,
    name text NOT NULL,
    value text NOT NULL,
    visibility_policy text NOT NULL DEFAULT 'private',
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT actions_variables_scope_kind_check CHECK (scope_kind IN ('repository', 'organization', 'environment')),
    CONSTRAINT actions_variables_visibility_policy_check CHECK (visibility_policy IN ('private', 'selected_repositories', 'all_repositories')),
    CONSTRAINT actions_variables_name_identifier CHECK (name ~ '^[A-Z_][A-Z0-9_]*$'),
    CONSTRAINT actions_variables_scope_name_required CHECK (scope_kind = 'repository' OR length(trim(COALESCE(scope_name, ''))) > 0)
);

CREATE UNIQUE INDEX actions_variables_repository_scope_name_unique
ON actions_variables (repository_id, scope_kind, COALESCE(scope_name, ''), name);

CREATE INDEX actions_variables_repository_updated_idx
ON actions_variables (repository_id, updated_at DESC);

CREATE TRIGGER actions_variables_set_updated_at
BEFORE UPDATE ON actions_variables
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
