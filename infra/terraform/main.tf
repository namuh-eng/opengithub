locals {
  name                = "${var.project}-${var.environment}"
  ses_identity_domain = var.ses_identity_domain != "" ? var.ses_identity_domain : var.domain_name
  api_image           = var.api_image != "" ? var.api_image : "public.ecr.aws/docker/library/nginx:stable-alpine"
  web_image           = var.web_image != "" ? var.web_image : "public.ecr.aws/docker/library/nginx:stable-alpine"
  migration_image     = var.migration_image != "" ? var.migration_image : local.api_image
  worker_enabled      = var.worker_image != "" && var.worker_desired_count > 0
  ses_from_address    = var.ses_from_address != "" ? var.ses_from_address : "noreply@${local.ses_identity_domain}"
  tags = {
    Project     = var.project
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

data "aws_region" "current" {}

resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true
  tags                 = { Name = local.name }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = local.name }
}

resource "aws_subnet" "public" {
  count                   = var.az_count
  vpc_id                  = aws_vpc.main.id
  cidr_block              = cidrsubnet(var.vpc_cidr, 8, count.index)
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true
  tags                    = { Name = "${local.name}-public-${count.index + 1}", Tier = "public" }
}

resource "aws_subnet" "private" {
  count             = var.az_count
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 10)
  availability_zone = data.aws_availability_zones.available.names[count.index]
  tags              = { Name = "${local.name}-private-${count.index + 1}", Tier = "private" }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }
  tags = { Name = "${local.name}-public" }
}

resource "aws_route_table_association" "public" {
  count          = var.az_count
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_eip" "nat" {
  count  = var.enable_nat_gateway ? var.az_count : 0
  domain = "vpc"
  tags   = { Name = "${local.name}-nat-${count.index + 1}" }
}

resource "aws_nat_gateway" "main" {
  count         = var.enable_nat_gateway ? var.az_count : 0
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  tags          = { Name = "${local.name}-nat-${count.index + 1}" }
  depends_on    = [aws_internet_gateway.main]
}

resource "aws_route_table" "private" {
  count  = var.az_count
  vpc_id = aws_vpc.main.id
  dynamic "route" {
    for_each = var.enable_nat_gateway ? [1] : []
    content {
      cidr_block     = "0.0.0.0/0"
      nat_gateway_id = aws_nat_gateway.main[count.index].id
    }
  }
  tags = { Name = "${local.name}-private-${count.index + 1}" }
}

resource "aws_route_table_association" "private" {
  count          = var.az_count
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[count.index].id
}

resource "aws_security_group" "alb" {
  name        = "${local.name}-alb"
  description = "Public ALB ingress"
  vpc_id      = aws_vpc.main.id
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = var.allowed_http_cidrs
  }
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = var.allowed_http_cidrs
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "ecs" {
  name        = "${local.name}-ecs"
  description = "ECS tasks; only ALB may call app ports"
  vpc_id      = aws_vpc.main.id
  ingress {
    from_port       = 3000
    to_port         = 3000
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "rds" {
  name        = "${local.name}-rds"
  description = "Private Postgres; ECS only"
  vpc_id      = aws_vpc.main.id
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.ecs.id]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_ecr_repository" "api" {
  name                 = "${local.name}/api"
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration { scan_on_push = true }
}

resource "aws_ecr_repository" "web" {
  name                 = "${local.name}/web"
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration { scan_on_push = true }
}

resource "aws_ecr_repository" "worker" {
  name                 = "${local.name}/worker"
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration { scan_on_push = true }
}

resource "aws_ecr_repository" "migration" {
  name                 = "${local.name}/migration"
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration { scan_on_push = true }
}

resource "aws_s3_bucket" "storage" {
  bucket = "${local.name}-storage-${data.aws_caller_identity.current.account_id}"
}

resource "aws_s3_bucket_public_access_block" "storage" {
  bucket                  = aws_s3_bucket.storage.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "storage" {
  bucket = aws_s3_bucket.storage.id
  versioning_configuration { status = "Enabled" }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "storage" {
  bucket = aws_s3_bucket.storage.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_db_subnet_group" "main" {
  name       = local.name
  subnet_ids = aws_subnet.private[*].id
}

resource "random_password" "db" {
  length  = 32
  special = false
}

resource "aws_secretsmanager_secret" "database_url" {
  name        = "/${local.name}/DATABASE_URL"
  description = "Postgres connection string for OpenGitHub ECS tasks"
}

resource "aws_db_instance" "postgres" {
  identifier                 = local.name
  engine                     = "postgres"
  engine_version             = "16"
  instance_class             = var.rds_instance_class
  allocated_storage          = var.rds_allocated_storage
  max_allocated_storage      = max(var.rds_allocated_storage * 2, 100)
  db_name                    = "opengithub"
  username                   = "opengithub"
  password                   = random_password.db.result
  db_subnet_group_name       = aws_db_subnet_group.main.name
  vpc_security_group_ids     = [aws_security_group.rds.id]
  publicly_accessible        = false
  storage_encrypted          = true
  backup_retention_period    = var.rds_backup_retention_days
  deletion_protection        = var.rds_deletion_protection
  skip_final_snapshot        = false
  final_snapshot_identifier  = "${local.name}-final"
  auto_minor_version_upgrade = true
}

resource "aws_secretsmanager_secret_version" "database_url" {
  secret_id     = aws_secretsmanager_secret.database_url.id
  secret_string = "postgresql://${aws_db_instance.postgres.username}:${random_password.db.result}@${aws_db_instance.postgres.address}:5432/${aws_db_instance.postgres.db_name}"
}

resource "aws_secretsmanager_secret" "app" {
  for_each = toset(["SESSION_SECRET", "AUTH_GOOGLE_ID", "AUTH_GOOGLE_SECRET", "OPENAI_API_KEY"])
  name     = "/${local.name}/${each.key}"
}

resource "aws_ssm_parameter" "storage_bucket" {
  name  = "/${local.name}/S3_BUCKET"
  type  = "String"
  value = aws_s3_bucket.storage.bucket
}

resource "aws_ssm_parameter" "ses_domain" {
  name  = "/${local.name}/SES_DOMAIN"
  type  = "String"
  value = local.ses_identity_domain
}

resource "aws_ssm_parameter" "ses_from_address" {
  name  = "/${local.name}/EMAIL_FROM_ADDRESS"
  type  = "String"
  value = local.ses_from_address
}

resource "aws_ssm_parameter" "ses_configuration_set" {
  count = var.ses_configuration_set == "" ? 0 : 1
  name  = "/${local.name}/SES_CONFIGURATION_SET"
  type  = "String"
  value = var.ses_configuration_set
}

resource "aws_cloudwatch_log_group" "app" {
  for_each          = toset(["api", "web", "worker", "migration"])
  name              = "/ecs/${local.name}/${each.key}"
  retention_in_days = 30
}

resource "aws_iam_role" "execution" {
  name               = "${local.name}-ecs-execution"
  assume_role_policy = jsonencode({ Version = "2012-10-17", Statement = [{ Effect = "Allow", Principal = { Service = "ecs-tasks.amazonaws.com" }, Action = "sts:AssumeRole" }] })
}

resource "aws_iam_role_policy_attachment" "execution" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy" "execution_secrets" {
  role = aws_iam_role.execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["secretsmanager:GetSecretValue", "ssm:GetParameters", "ssm:GetParameter"]
      Resource = concat([aws_secretsmanager_secret.database_url.arn], [for s in aws_secretsmanager_secret.app : s.arn], [aws_ssm_parameter.storage_bucket.arn, aws_ssm_parameter.ses_domain.arn, aws_ssm_parameter.ses_from_address.arn], [for p in aws_ssm_parameter.ses_configuration_set : p.arn])
    }]
  })
}

resource "aws_iam_role" "api_task" {
  name               = "${local.name}-api-task"
  assume_role_policy = jsonencode({ Version = "2012-10-17", Statement = [{ Effect = "Allow", Principal = { Service = "ecs-tasks.amazonaws.com" }, Action = "sts:AssumeRole" }] })
}

resource "aws_iam_role" "web_task" {
  name               = "${local.name}-web-task"
  assume_role_policy = aws_iam_role.api_task.assume_role_policy
}

resource "aws_iam_role" "worker_task" {
  name               = "${local.name}-worker-task"
  assume_role_policy = aws_iam_role.api_task.assume_role_policy
}

resource "aws_iam_role" "migration_task" {
  name               = "${local.name}-migration-task"
  assume_role_policy = aws_iam_role.api_task.assume_role_policy
}

resource "aws_iam_role_policy" "api_task" {
  role = aws_iam_role.api_task.id
  policy = jsonencode({ Version = "2012-10-17", Statement = [
    { Effect = "Allow", Action = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"], Resource = "${aws_s3_bucket.storage.arn}/api/*" },
    { Effect = "Allow", Action = ["ses:SendEmail"], Resource = aws_ses_domain_identity.main.arn, Condition = { StringLike = { "ses:FromAddress" = local.ses_from_address } } }
  ] })
}

resource "aws_iam_role_policy" "worker_task" {
  role = aws_iam_role.worker_task.id
  policy = jsonencode({ Version = "2012-10-17", Statement = [
    { Effect = "Allow", Action = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject", "s3:ListBucket"], Resource = [aws_s3_bucket.storage.arn, "${aws_s3_bucket.storage.arn}/worker/*"] },
    { Effect = "Allow", Action = ["ses:SendEmail"], Resource = aws_ses_domain_identity.main.arn, Condition = { StringLike = { "ses:FromAddress" = local.ses_from_address } } }
  ] })
}

resource "aws_lb" "main" {
  name               = local.name
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = aws_subnet.public[*].id
}

resource "aws_lb_target_group" "api" {
  name        = "${local.name}-api"
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = aws_vpc.main.id
  target_type = "ip"
  health_check {
    path    = "/health"
    matcher = "200-399"
  }
}

resource "aws_lb_target_group" "web" {
  name        = "${local.name}-web"
  port        = 3000
  protocol    = "HTTP"
  vpc_id      = aws_vpc.main.id
  target_type = "ip"
  health_check {
    path    = "/"
    matcher = "200-399"
  }
}

resource "aws_acm_certificate" "alb" {
  domain_name       = var.domain_name
  validation_method = "DNS"
  lifecycle { create_before_destroy = true }
}

resource "aws_acm_certificate" "cloudfront" {
  provider          = aws.us_east_1
  domain_name       = var.domain_name
  validation_method = "DNS"
  lifecycle { create_before_destroy = true }
}

resource "aws_route53_record" "alb_validation" {
  for_each        = var.route53_zone_id == "" ? {} : { for dvo in aws_acm_certificate.alb.domain_validation_options : dvo.domain_name => dvo }
  zone_id         = var.route53_zone_id
  name            = each.value.resource_record_name
  type            = each.value.resource_record_type
  records         = [each.value.resource_record_value]
  ttl             = 60
  allow_overwrite = true
}

resource "aws_route53_record" "cloudfront_validation" {
  for_each        = var.route53_zone_id == "" ? {} : { for dvo in aws_acm_certificate.cloudfront.domain_validation_options : dvo.domain_name => dvo }
  zone_id         = var.route53_zone_id
  name            = each.value.resource_record_name
  type            = each.value.resource_record_type
  records         = [each.value.resource_record_value]
  ttl             = 60
  allow_overwrite = true
}

resource "aws_acm_certificate_validation" "alb" {
  count                   = var.route53_zone_id == "" ? 0 : 1
  certificate_arn         = aws_acm_certificate.alb.arn
  validation_record_fqdns = [for record in aws_route53_record.alb_validation : record.fqdn]
}

resource "aws_acm_certificate_validation" "cloudfront" {
  count                   = var.route53_zone_id == "" ? 0 : 1
  provider                = aws.us_east_1
  certificate_arn         = aws_acm_certificate.cloudfront.arn
  validation_record_fqdns = [for record in aws_route53_record.cloudfront_validation : record.fqdn]
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.main.arn
  port              = 80
  protocol          = "HTTP"
  default_action {
    type = "redirect"
    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.main.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.route53_zone_id == "" ? aws_acm_certificate.alb.arn : aws_acm_certificate_validation.alb[0].certificate_arn
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.web.arn
  }
}

resource "aws_lb_listener_rule" "api" {
  listener_arn = aws_lb_listener.https.arn
  priority     = 10
  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.api.arn
  }
  condition {
    path_pattern {
      values = ["/api/*", "/git/*", "/health"]
    }
  }
}

resource "aws_ecs_cluster" "main" { name = local.name }


resource "aws_ecs_task_definition" "migration" {
  family                   = "${local.name}-migration"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = var.migration_cpu
  memory                   = var.migration_memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.migration_task.arn
  container_definitions = jsonencode([{
    name      = "migration"
    image     = local.migration_image
    essential = true
    command   = ["sqlx", "migrate", "run", "--source", "crates/api/migrations"]
    secrets   = [{ name = "DATABASE_URL", valueFrom = aws_secretsmanager_secret.database_url.arn }]
    environment = [
      { name = "AWS_REGION", value = var.aws_region },
      { name = "SQLX_MIGRATIONS_SOURCE", value = "crates/api/migrations" }
    ]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        awslogs-group         = aws_cloudwatch_log_group.app["migration"].name
        awslogs-region        = var.aws_region
        awslogs-stream-prefix = "ecs"
      }
    }
  }])
}

resource "aws_ecs_task_definition" "api" {
  family                   = "${local.name}-api"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = var.task_cpu
  memory                   = var.task_memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.api_task.arn
  container_definitions    = jsonencode([{ name = "api", image = local.api_image, essential = true, portMappings = [{ containerPort = 8080, protocol = "tcp" }], secrets = concat([{ name = "DATABASE_URL", valueFrom = aws_secretsmanager_secret.database_url.arn }], [for k, s in aws_secretsmanager_secret.app : { name = k, valueFrom = s.arn }]), environment = [{ name = "AWS_REGION", value = var.aws_region }, { name = "S3_BUCKET", value = aws_s3_bucket.storage.bucket }, { name = "SES_DOMAIN", value = local.ses_identity_domain }, { name = "EMAIL_DELIVERY_PROVIDER", value = "ses" }, { name = "EMAIL_FROM_ADDRESS", value = local.ses_from_address }, { name = "SES_CONFIGURATION_SET", value = var.ses_configuration_set }], logConfiguration = { logDriver = "awslogs", options = { awslogs-group = aws_cloudwatch_log_group.app["api"].name, awslogs-region = var.aws_region, awslogs-stream-prefix = "ecs" } } }])
}

resource "aws_ecs_task_definition" "web" {
  family                   = "${local.name}-web"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = var.task_cpu
  memory                   = var.task_memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.web_task.arn
  container_definitions    = jsonencode([{ name = "web", image = local.web_image, essential = true, portMappings = [{ containerPort = 3000, protocol = "tcp" }], environment = [{ name = "NEXT_PUBLIC_API_URL", value = "https://${var.domain_name}" }], logConfiguration = { logDriver = "awslogs", options = { awslogs-group = aws_cloudwatch_log_group.app["web"].name, awslogs-region = var.aws_region, awslogs-stream-prefix = "ecs" } } }])
}

resource "aws_ecs_task_definition" "worker" {
  count                    = local.worker_enabled ? 1 : 0
  family                   = "${local.name}-worker"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = var.worker_cpu
  memory                   = var.worker_memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.worker_task.arn
  container_definitions    = jsonencode([{ name = "worker", image = var.worker_image, essential = true, secrets = [{ name = "DATABASE_URL", valueFrom = aws_secretsmanager_secret.database_url.arn }], environment = [{ name = "AWS_REGION", value = var.aws_region }, { name = "S3_BUCKET", value = aws_s3_bucket.storage.bucket }, { name = "SES_DOMAIN", value = local.ses_identity_domain }, { name = "EMAIL_DELIVERY_PROVIDER", value = "ses" }, { name = "EMAIL_FROM_ADDRESS", value = local.ses_from_address }, { name = "SES_CONFIGURATION_SET", value = var.ses_configuration_set }], logConfiguration = { logDriver = "awslogs", options = { awslogs-group = aws_cloudwatch_log_group.app["worker"].name, awslogs-region = var.aws_region, awslogs-stream-prefix = "ecs" } } }])
}

resource "aws_ecs_service" "api" {
  name            = "api"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.api.arn
  desired_count   = var.api_desired_count
  launch_type     = "FARGATE"
  network_configuration {
    subnets          = aws_subnet.private[*].id
    security_groups  = [aws_security_group.ecs.id]
    assign_public_ip = false
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.api.arn
    container_name   = "api"
    container_port   = 8080
  }
}

resource "aws_ecs_service" "web" {
  name            = "web"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.web.arn
  desired_count   = var.web_desired_count
  launch_type     = "FARGATE"
  network_configuration {
    subnets          = aws_subnet.private[*].id
    security_groups  = [aws_security_group.ecs.id]
    assign_public_ip = false
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.web.arn
    container_name   = "web"
    container_port   = 3000
  }
}

resource "aws_ecs_service" "worker" {
  count           = local.worker_enabled ? 1 : 0
  name            = "worker"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.worker[0].arn
  desired_count   = var.worker_desired_count
  launch_type     = "FARGATE"
  network_configuration {
    subnets          = aws_subnet.private[*].id
    security_groups  = [aws_security_group.ecs.id]
    assign_public_ip = false
  }
}

resource "aws_ses_domain_identity" "main" {
  domain = local.ses_identity_domain
}

resource "aws_ses_domain_dkim" "main" {
  domain = aws_ses_domain_identity.main.domain
}

resource "aws_ses_email_identity" "sender" {
  email = local.ses_from_address
}

resource "aws_route53_record" "ses_verification" {
  count   = var.route53_zone_id == "" ? 0 : 1
  zone_id = var.route53_zone_id
  name    = "_amazonses.${local.ses_identity_domain}"
  type    = "TXT"
  ttl     = 600
  records = [aws_ses_domain_identity.main.verification_token]
}

resource "aws_route53_record" "ses_dkim" {
  count   = var.route53_zone_id == "" ? 0 : 3
  zone_id = var.route53_zone_id
  name    = "${aws_ses_domain_dkim.main.dkim_tokens[count.index]}._domainkey.${local.ses_identity_domain}"
  type    = "CNAME"
  ttl     = 600
  records = ["${aws_ses_domain_dkim.main.dkim_tokens[count.index]}.dkim.amazonses.com"]
}

resource "aws_cloudfront_distribution" "main" {
  enabled             = true
  aliases             = [var.domain_name]
  default_root_object = ""
  origin {
    domain_name = aws_lb.main.dns_name
    origin_id   = "alb"
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }
  default_cache_behavior {
    allowed_methods        = ["GET", "HEAD", "OPTIONS", "PUT", "POST", "PATCH", "DELETE"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "alb"
    viewer_protocol_policy = "redirect-to-https"
    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Host"]
      cookies {
        forward = "all"
      }
    }
  }
  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }
  viewer_certificate {
    acm_certificate_arn      = var.route53_zone_id == "" ? aws_acm_certificate.cloudfront.arn : aws_acm_certificate_validation.cloudfront[0].certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }
}

resource "aws_route53_record" "app" {
  count   = var.route53_zone_id == "" ? 0 : 1
  zone_id = var.route53_zone_id
  name    = var.domain_name
  type    = "A"
  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_cloudwatch_metric_alarm" "alb_5xx" {
  alarm_name          = "${local.name}-alb-5xx"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "HTTPCode_ELB_5XX_Count"
  namespace           = "AWS/ApplicationELB"
  period              = 300
  statistic           = "Sum"
  threshold           = 10
  dimensions          = { LoadBalancer = aws_lb.main.arn_suffix }
}

resource "aws_cloudwatch_metric_alarm" "rds_cpu" {
  alarm_name          = "${local.name}-rds-cpu"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 3
  metric_name         = "CPUUtilization"
  namespace           = "AWS/RDS"
  period              = 300
  statistic           = "Average"
  threshold           = 80
  dimensions          = { DBInstanceIdentifier = aws_db_instance.postgres.id }
}
