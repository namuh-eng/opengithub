DROP TRIGGER IF EXISTS actions_variables_set_updated_at ON actions_variables;
DROP INDEX IF EXISTS actions_variables_repository_updated_idx;
DROP INDEX IF EXISTS actions_variables_repository_scope_name_unique;
DROP TABLE IF EXISTS actions_variables;

DROP TRIGGER IF EXISTS actions_secrets_set_updated_at ON actions_secrets;
DROP INDEX IF EXISTS actions_secrets_repository_updated_idx;
DROP INDEX IF EXISTS actions_secrets_repository_scope_name_unique;
DROP TABLE IF EXISTS actions_secrets;
