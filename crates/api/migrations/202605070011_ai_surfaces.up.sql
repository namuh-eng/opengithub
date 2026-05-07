ALTER TABLE user_settings
    ADD COLUMN IF NOT EXISTS ai_features_enabled boolean NOT NULL DEFAULT true;

ALTER TABLE repositories
    ADD COLUMN IF NOT EXISTS ai_features_enabled boolean NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS ai_outputs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    kind text NOT NULL,
    scope_type text NOT NULL,
    scope_id uuid NOT NULL,
    content_hash text NOT NULL,
    prompt_version text NOT NULL,
    model text NOT NULL,
    output text NOT NULL,
    generated_at timestamptz NOT NULL DEFAULT now(),
    regenerated_count integer NOT NULL DEFAULT 0,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT ai_outputs_kind_check CHECK (
        kind IN ('repo_summary', 'pr_summary', 'changelog')
    ),
    CONSTRAINT ai_outputs_scope_type_check CHECK (
        scope_type IN ('repository', 'pull_request', 'release')
    ),
    CONSTRAINT ai_outputs_content_hash_not_blank CHECK (length(trim(content_hash)) > 0),
    CONSTRAINT ai_outputs_prompt_version_not_blank CHECK (length(trim(prompt_version)) > 0),
    CONSTRAINT ai_outputs_model_not_blank CHECK (length(trim(model)) > 0),
    CONSTRAINT ai_outputs_output_not_blank CHECK (length(trim(output)) > 0),
    CONSTRAINT ai_outputs_regenerated_non_negative CHECK (regenerated_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ai_outputs_scope_cache_unique
ON ai_outputs (kind, scope_type, scope_id, content_hash, prompt_version, model);

CREATE INDEX IF NOT EXISTS ai_outputs_scope_latest_idx
ON ai_outputs (scope_type, scope_id, kind, generated_at DESC);
