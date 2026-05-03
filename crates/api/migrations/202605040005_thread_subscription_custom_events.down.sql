ALTER TABLE pull_request_subscriptions
DROP CONSTRAINT IF EXISTS pull_request_subscriptions_custom_events_check;

ALTER TABLE issue_subscriptions
DROP CONSTRAINT IF EXISTS issue_subscriptions_custom_events_check;

ALTER TABLE pull_request_subscriptions
DROP COLUMN IF EXISTS custom_events;

ALTER TABLE issue_subscriptions
DROP COLUMN IF EXISTS custom_events,
DROP COLUMN IF EXISTS subscribed;
