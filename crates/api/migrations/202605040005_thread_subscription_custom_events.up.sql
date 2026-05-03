ALTER TABLE issue_subscriptions
ADD COLUMN IF NOT EXISTS subscribed boolean NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS custom_events text[] NOT NULL DEFAULT '{}'::text[];

ALTER TABLE pull_request_subscriptions
ADD COLUMN IF NOT EXISTS custom_events text[] NOT NULL DEFAULT '{}'::text[];

ALTER TABLE issue_subscriptions
DROP CONSTRAINT IF EXISTS issue_subscriptions_custom_events_check;

ALTER TABLE issue_subscriptions
ADD CONSTRAINT issue_subscriptions_custom_events_check
CHECK (custom_events <@ ARRAY['closed', 'reopened', 'merged']::text[]);

ALTER TABLE pull_request_subscriptions
DROP CONSTRAINT IF EXISTS pull_request_subscriptions_custom_events_check;

ALTER TABLE pull_request_subscriptions
ADD CONSTRAINT pull_request_subscriptions_custom_events_check
CHECK (custom_events <@ ARRAY['closed', 'reopened', 'merged']::text[]);
