# Terraform configuration for Workforce OS MVP (AWS)
# NOTE: This is a minimal skeleton – replace placeholder values with real ones before applying.

terraform {
  required_version = ">= 1.3.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region for all resources"
  type        = string
  default     = "us-east-1"
}

# ---------------------------------------------------
# 1. API Gateway (REST) with Mutual TLS
# ---------------------------------------------------
resource "aws_api_gateway_rest_api" "telemetry_api" {
  name        = "workforce-telemetry-api"
  description = "Ingress endpoint for telemetry agents"
}

resource "aws_api_gateway_resource" "telemetry_resource" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_api.id
  parent_id   = aws_api_gateway_rest_api.telemetry_api.root_resource_id
  path_part   = "v1"
}

resource "aws_api_gateway_resource" "telemetry_ingress" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_api.id
  parent_id   = aws_api_gateway_resource.telemetry_resource.id
  path_part   = "telemetry"
}

resource "aws_api_gateway_resource" "ingress" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_api.id
  parent_id   = aws_api_gateway_resource.telemetry_ingress.id
  path_part   = "ingress"
}

resource "aws_api_gateway_method" "post_method" {
  rest_api_id   = aws_api_gateway_rest_api.telemetry_api.id
  resource_id    = aws_api_gateway_resource.ingress.id
  http_method    = "POST"
  authorization  = "NONE" # mTLS handled at domain level
}

resource "aws_api_gateway_integration" "lambda_integration" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_api.id
  resource_id  = aws_api_gateway_resource.ingress.id
  http_method  = aws_api_gateway_method.post_method.http_method
  type        = "MOCK" # Placeholder – replace with Kinesis/Kafka integration
  request_templates = {
    "application/json" = "{\"statusCode\": 202}"
  }
}

resource "aws_api_gateway_deployment" "api_deploy" {
  depends_on = [aws_api_gateway_integration.lambda_integration]
  rest_api_id = aws_api_gateway_rest_api.telemetry_api.id
  stage_name  = "prod"
}

# ---------------------------------------------------
# 2. MSK (Kafka) cluster – buffer for telemetry events
# ---------------------------------------------------
resource "aws_msk_cluster" "telemetry_cluster" {
  cluster_name           = "workforce-telemetry-cluster"
  kafka_version          = "2.8.1"
  number_of_broker_nodes = 3

  broker_node_group_info {
    instance_type = "kafka.m5.large"
    client_subnets = var.private_subnet_ids
    security_groups = [aws_security_group.msk_sg.id]
  }
}

resource "aws_security_group" "msk_sg" {
  name        = "msk-sg"
  description = "Security group for MSK"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 9092
    to_port     = 9092
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# ---------------------------------------------------
# 3. RDS PostgreSQL – tenant metadata & configuration
# ---------------------------------------------------
resource "aws_db_instance" "postgres" {
  identifier              = "workforce-metadata"
  engine                  = "postgres"
  engine_version          = "14"
  instance_class          = "db.t3.medium"
  allocated_storage       = 20
  name                    = "workforce"
  username                = var.db_username
  password                = var.db_password
  skip_final_snapshot     = true
  publicly_accessible     = false
  vpc_security_group_ids  = [aws_security_group.rds_sg.id]
  db_subnet_group_name    = aws_db_subnet_group.rds_subnet.id
}

resource "aws_security_group" "rds_sg" {
  name        = "rds-sg"
  description = "Allow inbound from EKS workers"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    security_groups = [aws_security_group.eks_worker_sg.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_db_subnet_group" "rds_subnet" {
  name       = "workforce-rds-subnet"
  subnet_ids = var.private_subnet_ids
}

# ---------------------------------------------------
# 4. EKS Cluster – processing pods (behaviour engine)
# ---------------------------------------------------
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "19.21.0"

  cluster_name    = "workforce-eks"
  cluster_version = "1.28"
  subnets         = var.private_subnet_ids
  vpc_id          = var.vpc_id

  node_groups = {
    workers = {
      desired_capacity = 3
      max_capacity     = 5
      min_capacity     = 1
      instance_type    = "t3.medium"
      ami_type         = "AL2_x86_64"
    }
  }
}

# ---------------------------------------------------
# 5. S3 bucket – long‑term cold storage & audit logs
# ---------------------------------------------------
resource "aws_s3_bucket" "audit_logs" {
  bucket = "workforce-telemetry-audit-${random_id.suffix.hex}"
  acl    = "private"
}

resource "random_id" "suffix" {
  byte_length = 4
}

# ---------------------------------------------------
# 6. (Optional) ClickHouse – self‑managed instance for time‑series data
#    For brevity we expose a placeholder EC2 instance; production should use a managed ClickHouse service.
# ---------------------------------------------------
resource "aws_instance" "clickhouse" {
  ami           = var.ami_id
  instance_type  = "t3.large"
  subnet_id      = element(var.private_subnet_ids, 0)
  vpc_security_group_ids = [aws_security_group.clickhouse_sg.id]
  tags = {
    Name = "clickhouse-node"
  }
}

resource "aws_security_group" "clickhouse_sg" {
  name        = "clickhouse-sg"
  description = "Allow traffic from EKS workers"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 8123
    to_port     = 8123
    protocol    = "tcp"
    security_groups = [aws_security_group.eks_worker_sg.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# ---------------------------------------------------
# 7. Variables (to be overridden via tfvars or environment)
# ---------------------------------------------------
variable "vpc_id" {}
variable "private_subnet_ids" {type = list(string)}
variable "vpc_cidr" {}
variable "db_username" {}
variable "db_password" {}
variable "ami_id" {}

# ---------------------------------------------------
# Outputs – useful for CI/CD pipelines
# ---------------------------------------------------
output "api_gateway_url" {
  value = aws_api_gateway_deployment.api_deploy.invoke_url
}

output "kafka_bootstrap_brokers" {
  value = aws_msk_cluster.telemetry_cluster.bootstrap_brokers_tls
}

output "rds_endpoint" {
  value = aws_db_instance.postgres.endpoint
}

output "eks_cluster_name" {
  value = module.eks.cluster_name
}
