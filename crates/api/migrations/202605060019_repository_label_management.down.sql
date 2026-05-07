DROP INDEX IF EXISTS discussion_labels_label_id_idx;
DROP INDEX IF EXISTS issue_labels_issue_label_idx;
DROP INDEX IF EXISTS labels_repository_name_count_idx;
DROP INDEX IF EXISTS repository_label_events_label_created_idx;
DROP INDEX IF EXISTS repository_label_events_repo_created_idx;
DROP TABLE IF EXISTS repository_label_events;
