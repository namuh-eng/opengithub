# Production deployment and rollback

`scripts/deploy.sh deploy staging` builds API/web Docker images, tags them with the current git SHA, pushes to ECR, runs SQLx migrations, registers ECS task definitions, updates API/web ECS services, waits for service stability, verifies API `/ready` and web `/healthz`, and optionally invalidates CloudFront.

Use `DRY_RUN=1` to verify command construction without live AWS credentials.

## Staging deploy

```bash
DRY_RUN=1 \
AWS_REGION=us-east-1 AWS_ACCOUNT_ID=123456789012 \
ECR_API_REPOSITORY=opengithub-api ECR_WEB_REPOSITORY=opengithub-web \
ECS_CLUSTER=opengithub-staging \
ECS_API_SERVICE=api ECS_WEB_SERVICE=web \
ECS_API_TASK_FAMILY=opengithub-api ECS_WEB_TASK_FAMILY=opengithub-web \
API_URL=https://api.staging.example.com WEB_URL=https://staging.example.com \
scripts/deploy.sh deploy staging
```

## Rollback

Roll back by supplying the last known-good task definitions from the previous ECS deployment or release record:

```bash
AWS_REGION=us-east-1 ECS_CLUSTER=opengithub-staging \
ECS_API_SERVICE=api ECS_WEB_SERVICE=web \
ROLLBACK_API_TASK_DEFINITION=arn:aws:ecs:us-east-1:123456789012:task-definition/opengithub-api:42 \
ROLLBACK_WEB_TASK_DEFINITION=arn:aws:ecs:us-east-1:123456789012:task-definition/opengithub-web:42 \
API_URL=https://api.staging.example.com WEB_URL=https://staging.example.com \
scripts/deploy.sh rollback staging
```

Deployment logs include `git_sha` and ECR image digests. The script exits non-zero on build, push, migration, ECS stability, or health-check failure.
