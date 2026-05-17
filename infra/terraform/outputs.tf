output "alb_url" {
  description = "Direct ALB URL."
  value       = "https://${aws_lb.main.dns_name}"
}

output "cloudfront_url" {
  description = "CloudFront distribution URL."
  value       = "https://${aws_cloudfront_distribution.main.domain_name}"
}

output "application_url" {
  description = "Primary application URL."
  value       = "https://${var.domain_name}"
}

output "ecr_repositories" {
  description = "ECR repository URLs for image publishing. Use digest-pinned images in tfvars."
  value = {
    api    = aws_ecr_repository.api.repository_url
    web    = aws_ecr_repository.web.repository_url
    worker    = aws_ecr_repository.worker.repository_url
    migration = aws_ecr_repository.migration.repository_url
  }
}

output "secret_names" {
  description = "Secrets/parameters that must be populated before production traffic."
  value = concat(
    [aws_secretsmanager_secret.database_url.name],
    [for s in aws_secretsmanager_secret.app : s.name],
    [aws_ssm_parameter.storage_bucket.name, aws_ssm_parameter.ses_domain.name]
  )
}

output "rds_endpoint" {
  description = "Private RDS endpoint; only ECS security group can connect."
  value       = aws_db_instance.postgres.address
  sensitive   = true
}

output "ses_dns_records" {
  description = "SES TXT/DKIM values to create manually when route53_zone_id is empty."
  value = {
    verification_txt_name  = "_amazonses.${local.ses_identity_domain}"
    verification_txt_value = aws_ses_domain_identity.main.verification_token
    dkim_tokens            = aws_ses_domain_dkim.main.dkim_tokens
  }
}


output "migration_run_task" {
  description = "Inputs for the pre-rollout ECS RunTask that applies SQLx migrations against RDS."
  value = {
    cluster             = aws_ecs_cluster.main.name
    task_definition     = aws_ecs_task_definition.migration.arn
    container_name      = "migration"
    subnet_ids          = aws_subnet.private[*].id
    security_group_ids  = [aws_security_group.ecs.id]
    log_group           = aws_cloudwatch_log_group.app["migration"].name
    command             = "sqlx migrate run --source crates/api/migrations"
    database_secret_arn = aws_secretsmanager_secret.database_url.arn
  }
  sensitive = true
}
