variable "project" {
  description = "Short project name used in resource names."
  type        = string
  default     = "opengithub"
}

variable "environment" {
  description = "Environment name, for example staging or production."
  type        = string
  default     = "staging"
}

variable "aws_region" {
  description = "AWS region for regional resources."
  type        = string
  default     = "us-west-2"
}

variable "domain_name" {
  description = "Primary application domain, for example opengithub.example.com."
  type        = string
}

variable "route53_zone_id" {
  description = "Optional Route 53 hosted zone ID. When set, DNS validation records and app aliases are created."
  type        = string
  default     = ""
}

variable "ses_identity_domain" {
  description = "Domain to verify for SES. Defaults to domain_name."
  type        = string
  default     = ""
}

variable "vpc_cidr" {
  description = "CIDR block for the VPC."
  type        = string
  default     = "10.42.0.0/16"
}

variable "az_count" {
  description = "Number of availability zones/subnet pairs to create."
  type        = number
  default     = 2
}

variable "api_image" {
  description = "API container image URI, preferably pinned by digest. Leave empty for initial infra apply."
  type        = string
  default     = ""
}

variable "web_image" {
  description = "Web container image URI, preferably pinned by digest. Leave empty for initial infra apply."
  type        = string
  default     = ""
}

variable "worker_image" {
  description = "Optional worker image URI pinned by digest. Empty disables the worker service."
  type        = string
  default     = ""
}

variable "api_desired_count" {
  type    = number
  default = 1
}
variable "web_desired_count" {
  type    = number
  default = 1
}
variable "worker_desired_count" {
  type    = number
  default = 0
}

variable "task_cpu" {
  description = "CPU units for API and web Fargate tasks."
  type        = number
  default     = 512
}

variable "task_memory" {
  description = "Memory MiB for API and web Fargate tasks."
  type        = number
  default     = 1024
}

variable "worker_cpu" {
  type    = number
  default = 512
}
variable "worker_memory" {
  type    = number
  default = 1024
}

variable "rds_instance_class" {
  description = "RDS Postgres instance class."
  type        = string
  default     = "db.t4g.micro"
}

variable "rds_allocated_storage" {
  description = "Initial RDS storage in GiB."
  type        = number
  default     = 20
}

variable "rds_backup_retention_days" {
  description = "RDS automated backup retention days."
  type        = number
  default     = 7
}

variable "rds_deletion_protection" {
  description = "Protect RDS from accidental deletion. Keep true outside throwaway test accounts."
  type        = bool
  default     = true
}

variable "enable_nat_gateway" {
  description = "Create NAT gateways so private ECS tasks can pull images and reach AWS APIs."
  type        = bool
  default     = true
}

variable "allowed_http_cidrs" {
  description = "CIDRs allowed to reach ALB HTTP/HTTPS."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}
