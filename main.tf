// Terraform configuration for Telemetry Ingress infrastructure

terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "ap-southeast-2"
}

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "ap-southeast-2"
}

// Variable for customizable API Gateway stage name
variable "stage_name" {
  description = "Stage name for API Gateway deployment"
  type        = string
  default     = "prod"
}

# ---------------------------------------------------
# 1. VPC and Networking
# ---------------------------------------------------
resource "aws_vpc" "telemetry_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags = {
    Environment = "Production"
    Service     = "TelemetryIngress"
  }
}

resource "aws_subnet" "private_subnet_a" {
  vpc_id            = aws_vpc.telemetry_vpc.id
  cidr_block        = "10.0.1.0/24"
  availability_zone = "ap-southeast-2a"
  tags = {
    Name = "telemetry-private-a"
  }
}

# ---------------------------------------------------
# 2. Data Buffer (Amazon Kinesis)
# ---------------------------------------------------
resource "aws_kinesis_stream" "telemetry_raw_stream" {
  name             = "telemetry-ingress-stream"
  shard_count      = 10
  retention_period = 24

  stream_mode_details {
    stream_mode = "PROVISIONED"
  }
}

resource "aws_iam_role" "api_gateway_kinesis_role" {
  name = "api-gateway-kinesis-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Action = "sts:AssumeRole",
        Effect = "Allow",
        Principal = {
          Service = "apigateway.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "api_gateway_kinesis_policy" {
  name = "api-gateway-kinesis-policy"
  role = aws_iam_role.api_gateway_kinesis_role.id

  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = "kinesis:PutRecord",
        Resource = aws_kinesis_stream.telemetry_raw_stream.arn
      }
    ]
  })
}

# ---------------------------------------------------
# 3. API Gateway (REST) with mTLS
# ---------------------------------------------------
resource "aws_api_gateway_rest_api" "telemetry_edge" {
  name        = "telemetry-edge-gateway"
  description = "Ingress for Darwin and Win32 endpoint agents"
  endpoint_configuration {
    types = ["REGIONAL"]
  }
  disable_execute_api_endpoint = true
}

# Domain name for mTLS (placeholder values – replace with real bucket/key)
resource "aws_api_gateway_domain_name" "mtls_domain" {
  domain_name = "telemetry.example.com"
  security_policy = "TLS_1_2"

  endpoint_configuration {
    types = ["REGIONAL"]
  }

  mutual_tls_authentication {
    truststore_uri     = "s3://my-mtls-bucket/truststore.pem"
    truststore_version = "v1"
  }
}

# Base‑path mapping to attach the API to the custom domain
resource "aws_api_gateway_base_path_mapping" "mtls_mapping" {
  api_id      = aws_api_gateway_rest_api.telemetry_edge.id
  stage_name  = var.stage_name
  domain_name = aws_api_gateway_domain_name.mtls_domain.domain_name
}

# Stage deployment (required for mapping)
// Deployment is handled by stage resource; keep for compatibility but no stage_name
resource "aws_api_gateway_deployment" "telemetry_edge_deploy" {
  depends_on = [aws_api_gateway_method.post_method, aws_api_gateway_integration.kinesis_integration]
  rest_api_id = aws_api_gateway_rest_api.telemetry_edge.id
}

resource "aws_api_gateway_stage" "telemetry_stage" {
  stage_name    = var.stage_name
  rest_api_id   = aws_api_gateway_rest_api.telemetry_edge.id
  deployment_id = aws_api_gateway_deployment.telemetry_edge_deploy.id
  description   = "Production stage"
}


# ---------------------------------------------------
# 4. Request & Response Model (Schema validation)
# ---------------------------------------------------
resource "aws_api_gateway_model" "telemetry_schema" {
  rest_api_id  = aws_api_gateway_rest_api.telemetry_edge.id
  name         = "TelemetryIngressModelV1_1"
  description  = "Validates incoming payloads against v1.1 schema (Org Hierarchy Supported)"
  content_type = "application/json"
  schema       = file("${path.module}/schemas/telemetry_ingress_v1_1.json")
}

resource "aws_api_gateway_request_validator" "strict_validator" {
  name                        = "strict-body-validator"
  rest_api_id                 = aws_api_gateway_rest_api.telemetry_edge.id
  validate_request_body       = true
  validate_request_parameters = false
}

# ---------------------------------------------------
# 5. Method & Integration (placeholder – actual integration can be added later)
# ---------------------------------------------------
resource "aws_api_gateway_resource" "telemetry_resource" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_edge.id
  parent_id   = aws_api_gateway_rest_api.telemetry_edge.root_resource_id
  path_part   = "telemetry"
}

resource "aws_api_gateway_method" "post_method" {
  rest_api_id   = aws_api_gateway_rest_api.telemetry_edge.id
  resource_id   = aws_api_gateway_resource.telemetry_resource.id
  http_method   = "POST"
  authorization = "NONE"
  request_validator_id = aws_api_gateway_request_validator.strict_validator.id
  request_models = {
    "application/json" = aws_api_gateway_model.telemetry_schema.name
  }
}

resource "aws_api_gateway_integration" "kinesis_integration" {
  rest_api_id             = aws_api_gateway_rest_api.telemetry_edge.id
  resource_id             = aws_api_gateway_resource.telemetry_resource.id
  http_method             = aws_api_gateway_method.post_method.http_method
  integration_http_method = "POST"
  type                    = "AWS"
  credentials             = aws_iam_role.api_gateway_kinesis_role.arn
  uri                     = "arn:aws:apigateway:${var.aws_region}:kinesis:action/PutRecord"

  request_templates = {
    "application/json" = <<EOF
{
  "StreamName": "${aws_kinesis_stream.telemetry_raw_stream.name}",
  "Data": "$util.base64Encode($input.json('$'))",
  "PartitionKey": "$input.path('$.tenant_id')"
}
EOF
  }
}

resource "aws_api_gateway_method_response" "response_200" {
  rest_api_id = aws_api_gateway_rest_api.telemetry_edge.id
  resource_id = aws_api_gateway_resource.telemetry_resource.id
  http_method = aws_api_gateway_method.post_method.http_method
  status_code = "200"
}

resource "aws_api_gateway_integration_response" "integration_response_200" {
  depends_on  = [aws_api_gateway_integration.kinesis_integration]
  rest_api_id = aws_api_gateway_rest_api.telemetry_edge.id
  resource_id = aws_api_gateway_resource.telemetry_resource.id
  http_method = aws_api_gateway_method.post_method.http_method
  status_code = aws_api_gateway_method_response.response_200.status_code
  selection_pattern = ""
}

# ---------------------------------------------------
# 6. Outputs (useful for CI checks)
# ---------------------------------------------------
output "api_gateway_url" {
  description = "Invoke URL for the API Gateway"
  value       = aws_api_gateway_stage.telemetry_stage.invoke_url
}

output "kinesis_stream_arn" {
  description = "ARN of the Kinesis Data Stream"
  value       = aws_kinesis_stream.telemetry_raw_stream.arn
}
