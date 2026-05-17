# OpenGitHub AWS Terraform

This directory provisions the AWS-only production shape required by issue #2. It is intentionally self-contained so a staging environment can be created from scratch before application images exist, then updated later by digest-pinned ECS task images.

## What it creates

- VPC with public ALB subnets and private ECS/RDS subnets.
- Public ALB with HTTP-to-HTTPS redirect, HTTPS listener, and `/api/*`, `/git/*`, `/health` routing to the API target group.
- ECS Fargate cluster, services, task definitions for `api`, `web`, optional `worker`, and a one-off SQLx migration task.
- ECR repositories for API, web, worker, and SQLx migration images with immutable tags and scan-on-push.
- Private RDS Postgres 16 with encryption, backups, deletion protection, and an ingress rule that only allows the ECS task security group.
- Private S3 storage bucket with public access blocked, versioning, encryption, and per-role prefix-scoped IAM.
- SES domain identity, verified sender, and DKIM records; Route 53 records are created when `route53_zone_id` is set.
- Regional ACM certificate for the ALB and us-east-1 ACM certificate for CloudFront.
- CloudFront distribution in front of the ALB and optional Route 53 app alias.
- Secrets Manager/SSM entries for runtime configuration and outputs listing required secret names.
- CloudWatch log groups and core ALB/RDS alarms.

## First staging apply

```bash
cd infra/terraform
cp environments/staging/staging.tfvars.example staging.tfvars
# edit domain_name and, if AWS Route 53 hosts the zone, route53_zone_id
terraform init
terraform plan -var-file=staging.tfvars
terraform apply -var-file=staging.tfvars
```

If `route53_zone_id` is not supplied, Terraform still creates ACM and SES identities but you must add the printed DNS validation records manually before HTTPS/SES become active.

The first apply uses public nginx images when `api_image`/`web_image` are empty. This lets networking, RDS, S3, DNS, and certificates be created before the OpenGitHub images are published.

## Updating ECS by image digest

Build and push images to the output ECR repositories, then pin each image by digest in your tfvars:

```hcl
api_image = "123456789012.dkr.ecr.us-west-2.amazonaws.com/opengithub-staging/api@sha256:<digest>"
web_image = "123456789012.dkr.ecr.us-west-2.amazonaws.com/opengithub-staging/web@sha256:<digest>"
worker_image = "123456789012.dkr.ecr.us-west-2.amazonaws.com/opengithub-staging/worker@sha256:<digest>"
migration_image = "123456789012.dkr.ecr.us-west-2.amazonaws.com/opengithub-staging/migration@sha256:<digest>"
```

Run `terraform apply -var-file=staging.tfvars`. Terraform registers new task definitions and updates the ECS services.


## Database migration gate

Build `Dockerfile.migration`, push it to the `migration` ECR repository, and pin `migration_image` by digest before rolling ECS services. The task definition injects `DATABASE_URL` from Secrets Manager and runs:

```bash
sqlx migrate run --source crates/api/migrations
```

Use `terraform output -json migration_run_task` to populate `ECS_CLUSTER`, `MIGRATION_TASK_DEFINITION`, `ECS_SUBNETS`, and `ECS_SECURITY_GROUPS`, then run `scripts/deploy.sh`. A failed migration task exits non-zero and blocks rollout; success/failure is visible in the migration CloudWatch log group. See `docs/deploy/database-migrations.md` for rollback boundaries.

## Required secret population

`terraform output secret_names` lists all Secrets Manager and SSM names. Populate the placeholder application secrets after initial apply:

- `SESSION_SECRET`
- `AUTH_GOOGLE_ID`
- `AUTH_GOOGLE_SECRET`
- `OPENAI_API_KEY`

`DATABASE_URL` and SSM storage/SES parameters are managed by Terraform. `EMAIL_FROM_ADDRESS` is generated from `ses_from_address` (default `noreply@ses_identity_domain`) and injected into API/worker tasks along with `EMAIL_DELIVERY_PROVIDER=ses`.

## Validation

Use the repo wrapper:

```bash
./scripts/preflight.sh validate
```

It runs `terraform fmt -check`, `terraform init -backend=false`, and `terraform validate` when Terraform/OpenTofu is installed. Without Terraform it performs static file checks and exits non-zero for real provisioning actions.
