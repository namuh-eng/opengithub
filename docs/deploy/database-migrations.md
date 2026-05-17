# Production database migrations

OpenGitHub deploys schema changes before ECS service rollout by running a one-off Fargate task from `Dockerfile.migration`.

The migration container contains `sqlx-cli` and only the checked-in SQLx files under `crates/api/migrations/`. Its command is intentionally idempotent:

```bash
sqlx migrate run --source crates/api/migrations
```

SQLx records applied versions in the database, so re-running the task is safe after a successful or partially failed deployment.

## Staging / production rollout

1. Build and push the migration image, then pin it by digest in Terraform:

   ```hcl
   migration_image = "123456789012.dkr.ecr.us-west-2.amazonaws.com/opengithub-staging/migration@sha256:<digest>"
   ```

2. Apply Terraform so the migration task definition uses that digest.
3. Export `terraform output -json migration_run_task` values into the deploy environment:
   - `ECS_CLUSTER`
   - `MIGRATION_TASK_DEFINITION`
   - `ECS_SUBNETS` as a comma-separated list
   - `ECS_SECURITY_GROUPS` as a comma-separated list
4. Run `scripts/deploy.sh` before updating/waiting on ECS services. It starts the migration task, waits for it to stop, and fails the deploy if the migration container exits non-zero.

Migration logs are emitted to the Terraform output `migration_run_task.log_group` in CloudWatch with stream prefix `ecs/migration`.

## Rollback boundaries

Schema rollback is not automatic. Treat SQL migrations as forward-only unless a reversible down migration has been explicitly written, reviewed, and tested against a production snapshot.

If an application rollout fails after migrations succeeded:

1. Roll services back to the previous task definitions only if the old code is compatible with the migrated schema.
2. If the old code is not schema-compatible, roll forward with a hotfix that supports the new schema.
3. For destructive or incompatible schema changes, restore RDS from a point-in-time snapshot into a replacement database and repoint `DATABASE_URL`; do not run ad-hoc destructive SQL against production as a rollback shortcut.

Before merging destructive migrations, document the compatibility window and snapshot restore plan in the release notes.
