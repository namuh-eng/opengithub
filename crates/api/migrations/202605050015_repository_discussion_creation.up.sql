CREATE TABLE IF NOT EXISTS discussion_category_forms (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    category_id uuid NOT NULL REFERENCES discussion_categories(id) ON DELETE CASCADE,
    template_path text NOT NULL,
    title text,
    description text,
    body text NOT NULL DEFAULT '',
    fields jsonb NOT NULL DEFAULT '[]'::jsonb,
    valid boolean NOT NULL DEFAULT true,
    parse_error text,
    content_sha text NOT NULL DEFAULT '',
    parsed_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (repository_id, category_id),
    CONSTRAINT discussion_category_forms_path_not_blank CHECK (length(trim(template_path)) > 0),
    CONSTRAINT discussion_category_forms_fields_array CHECK (jsonb_typeof(fields) = 'array')
);

CREATE TABLE IF NOT EXISTS discussion_form_answers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    field_id text NOT NULL,
    field_label text NOT NULL,
    value text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (discussion_id, field_id),
    CONSTRAINT discussion_form_answers_field_id_not_blank CHECK (length(trim(field_id)) > 0),
    CONSTRAINT discussion_form_answers_label_not_blank CHECK (length(trim(field_label)) > 0)
);

CREATE TABLE IF NOT EXISTS discussion_subscriptions (
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    state text NOT NULL DEFAULT 'subscribed',
    reason text NOT NULL DEFAULT 'participating',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (discussion_id, user_id),
    CHECK (state IN ('subscribed', 'unsubscribed', 'ignored'))
);

CREATE TABLE IF NOT EXISTS discussion_attachments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid REFERENCES discussions(id) ON DELETE CASCADE,
    comment_id uuid REFERENCES discussion_comments(id) ON DELETE CASCADE,
    uploaded_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    file_name text NOT NULL,
    content_type text NOT NULL DEFAULT 'application/octet-stream',
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    status text NOT NULL DEFAULT 'draft',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (byte_size >= 0),
    CHECK (status IN ('draft', 'attached', 'deleted')),
    CONSTRAINT discussion_attachments_name_not_blank CHECK (length(trim(file_name)) > 0),
    CONSTRAINT discussion_attachments_storage_key_not_blank CHECK (length(trim(storage_key)) > 0)
);

CREATE INDEX IF NOT EXISTS discussion_category_forms_repository_category_idx
    ON discussion_category_forms(repository_id, category_id);
CREATE INDEX IF NOT EXISTS discussion_form_answers_discussion_idx
    ON discussion_form_answers(discussion_id, field_id);
CREATE INDEX IF NOT EXISTS discussion_subscriptions_user_idx
    ON discussion_subscriptions(user_id, state);
CREATE INDEX IF NOT EXISTS discussion_attachments_discussion_idx
    ON discussion_attachments(discussion_id, status);
