CREATE TABLE pull_request_file_hunks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    pull_request_file_id uuid NOT NULL REFERENCES pull_request_files(id) ON DELETE CASCADE,
    old_start bigint NOT NULL,
    old_lines bigint NOT NULL,
    new_start bigint NOT NULL,
    new_lines bigint NOT NULL,
    header text NOT NULL,
    display_order bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pull_request_file_hunks_old_start_positive CHECK (old_start >= 0),
    CONSTRAINT pull_request_file_hunks_old_lines_non_negative CHECK (old_lines >= 0),
    CONSTRAINT pull_request_file_hunks_new_start_positive CHECK (new_start >= 0),
    CONSTRAINT pull_request_file_hunks_new_lines_non_negative CHECK (new_lines >= 0),
    CONSTRAINT pull_request_file_hunks_display_order_non_negative CHECK (display_order >= 0)
);

CREATE INDEX pull_request_file_hunks_file_order_idx
ON pull_request_file_hunks (pull_request_file_id, display_order);

CREATE TABLE pull_request_hunk_lines (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    hunk_id uuid NOT NULL REFERENCES pull_request_file_hunks(id) ON DELETE CASCADE,
    kind text NOT NULL,
    old_line bigint,
    new_line bigint,
    content text NOT NULL DEFAULT '',
    position bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pull_request_hunk_lines_kind_check CHECK (kind IN ('context', 'added', 'removed')),
    CONSTRAINT pull_request_hunk_lines_position_non_negative CHECK (position >= 0),
    CONSTRAINT pull_request_hunk_lines_has_line CHECK (old_line IS NOT NULL OR new_line IS NOT NULL)
);

CREATE INDEX pull_request_hunk_lines_hunk_position_idx
ON pull_request_hunk_lines (hunk_id, position);

CREATE TABLE pull_request_viewed_files (
    pull_request_file_id uuid NOT NULL REFERENCES pull_request_files(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version_key text NOT NULL,
    viewed boolean NOT NULL DEFAULT true,
    viewed_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (pull_request_file_id, user_id),
    CONSTRAINT pull_request_viewed_files_version_not_blank CHECK (length(trim(version_key)) > 0)
);

CREATE INDEX pull_request_viewed_files_user_viewed_idx
ON pull_request_viewed_files (user_id, viewed_at DESC);

CREATE TRIGGER pull_request_viewed_files_set_updated_at
BEFORE UPDATE ON pull_request_viewed_files
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE pull_request_review_comments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    pull_request_id uuid NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    pull_request_file_id uuid NOT NULL REFERENCES pull_request_files(id) ON DELETE CASCADE,
    review_id uuid REFERENCES pull_request_reviews(id) ON DELETE SET NULL,
    author_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    body text NOT NULL,
    body_html text NOT NULL DEFAULT '',
    path text NOT NULL,
    side text NOT NULL DEFAULT 'right',
    old_line bigint,
    new_line bigint,
    position bigint,
    state text NOT NULL DEFAULT 'published',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pull_request_review_comments_body_not_blank CHECK (length(trim(body)) > 0),
    CONSTRAINT pull_request_review_comments_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT pull_request_review_comments_side_check CHECK (side IN ('left', 'right')),
    CONSTRAINT pull_request_review_comments_state_check CHECK (state IN ('pending', 'published', 'outdated', 'resolved')),
    CONSTRAINT pull_request_review_comments_has_line CHECK (old_line IS NOT NULL OR new_line IS NOT NULL)
);

CREATE INDEX pull_request_review_comments_pull_created_idx
ON pull_request_review_comments (pull_request_id, created_at);

CREATE INDEX pull_request_review_comments_file_line_idx
ON pull_request_review_comments (pull_request_file_id, side, old_line, new_line);

CREATE TRIGGER pull_request_review_comments_set_updated_at
BEFORE UPDATE ON pull_request_review_comments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE pull_request_review_drafts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    pull_request_id uuid NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    author_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    summary_body text,
    review_state text NOT NULL DEFAULT 'commented',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT pull_request_review_drafts_state_check CHECK (review_state IN ('commented', 'approved', 'changes_requested'))
);

CREATE UNIQUE INDEX pull_request_review_drafts_pull_author_unique
ON pull_request_review_drafts (pull_request_id, author_user_id);

CREATE TRIGGER pull_request_review_drafts_set_updated_at
BEFORE UPDATE ON pull_request_review_drafts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
