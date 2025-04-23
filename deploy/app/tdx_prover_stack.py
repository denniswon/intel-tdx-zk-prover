import aws_cdk as cdk
from aws_cdk import CfnOutput
from aws_cdk import Duration
from aws_cdk import Stack
from aws_cdk import aws_certificatemanager as acm
from aws_cdk import aws_ec2 as ec2
from aws_cdk import aws_ecr as ecr
from aws_cdk import aws_elasticloadbalancingv2 as elbv2
from aws_cdk import aws_iam as iam
from aws_cdk import aws_logs as logs
from aws_cdk import aws_lambda as _lambda
from aws_cdk import aws_secretsmanager as secretsmanager
from aws_cdk import aws_events as event_source
from aws_cdk import aws_events_targets as event_targets
from constructs import Construct


class TdxProver(Stack):
    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        deploy_env: str,
        app_shortname: str,
        db_security_group_id: str,
        git_commit: str,
        long_git_commit: str,
        vpc_id: str,
        aws_region: str,
        ecr_repository_arn: str,
        event_bus_arn: str,
        **kwargs,
    ) -> None:
        super().__init__(scope, construct_id, **kwargs)
        APP_SHORTNAME = app_shortname
        DOMAIN_NAME = "magiclabs.com"
        PRIMARY_REGION = "us-west-2"
        SHORT_REGION = aws_region.replace("-", "")[:4] + aws_region.replace("-", "")[-1]

        # Add tags to all resources in the stack
        cdk.Tags.of(self).add("project", app_shortname)
        cdk.Tags.of(self).add("stack", f"{app_shortname}-stack")

        vpc = ec2.Vpc.from_lookup(self, "ExistingVPC", vpc_id=vpc_id)

        # Create a load balancer
        lb = elbv2.ApplicationLoadBalancer(
            self,
            f"{APP_SHORTNAME}-lb",
            vpc=vpc,
            internet_facing=True,
            load_balancer_name=f"{APP_SHORTNAME}-lb",
        )

        # Create a target group
        target_group = elbv2.ApplicationTargetGroup(
            self,
            f"{APP_SHORTNAME}-tg",
            target_group_name=f"{APP_SHORTNAME}-tg",
            vpc=vpc,
            port=8002,
            protocol=elbv2.ApplicationProtocol.HTTP,
            target_type=elbv2.TargetType.IP,
            health_check=elbv2.HealthCheck(
                path="/api/health",
                healthy_threshold_count=2,
                unhealthy_threshold_count=2,
                timeout=cdk.Duration.seconds(2),
                interval=cdk.Duration.seconds(5),
                healthy_http_codes="200",
            ),
        )

        # Create a SSL certificate
        certificate = acm.Certificate(
            self,
            "SiteCertificate",
            domain_name=f"{APP_SHORTNAME}.{deploy_env}-{SHORT_REGION}.{DOMAIN_NAME}",
            validation=acm.CertificateValidation.from_dns(),
        )

        # Create a listener
        lb.add_listener(f"{APP_SHORTNAME}-listener", port=80, default_target_groups=[target_group])

        # Create HTTPS listener
        elbv2.ApplicationListener(
            self,
            f"{APP_SHORTNAME}_HttpsListener",
            load_balancer=lb,
            port=443,
            protocol=elbv2.ApplicationProtocol.HTTPS,
            certificates=[elbv2.ListenerCertificate.from_certificate_manager(certificate)],
            default_target_groups=[target_group],
        )

        # Reference the ECR repository in another account
        ecr_repo = ecr.Repository.from_repository_arn(
            self, "TdxProverEcrRepo", repository_arn=ecr_repository_arn
        )

        # Create a secret for service secrets seeded with key/value array for datadog
        self.service_secrets = secretsmanager.Secret(
            self,
            f"{APP_SHORTNAME}Secrets",
            description=f"{APP_SHORTNAME} secrets",
            secret_name=f"{APP_SHORTNAME}-secrets",
            generate_secret_string=secretsmanager.SecretStringGenerator(
                secret_string_template='{"DD_API_KEY": "", "DATABASE_URL": ""}',
                generate_string_key="DD_API_KEY",
                exclude_punctuation=True,
            ),
        )

        # Create a security group for ECS task
        task_security_group = ec2.SecurityGroup(
            self,
            f"{APP_SHORTNAME}TaskSecurityGroup",
            security_group_name=f"{APP_SHORTNAME}-task-sg",
            vpc=vpc,
            description=f"Security group for {APP_SHORTNAME} ECS task",
        )

        # Create a security group for the database
        db_security_group = ec2.SecurityGroup(
            self,
            f"TdxProverDbSecurityGroup-{aws_region}",
            vpc=vpc,
            description="Allow inbound traffic on port 5432 from ECS tasks",
            allow_all_outbound=True,
            security_group_name=f"tdx-prover-{aws_region}-db-sg",
        )
        db_security_group.add_ingress_rule(
            peer=ec2.Peer.security_group_id(task_security_group.security_group_id),
            connection=ec2.Port.tcp(5432),
            description=f"tdx-prover {aws_region}: Allow inbound traffic on port 5432 from ECS tasks",  # noqa: E501
        )

        # Look up existing db security
        existing_db_security_group = ec2.SecurityGroup.from_security_group_id(
            self,
            "ExistingDbSecurityGroup",
            db_security_group_id,
        )

        # Add an ingress rule to the existing security group
        existing_db_security_group.add_ingress_rule(
            peer=ec2.Peer.security_group_id(task_security_group.security_group_id),
            connection=ec2.Port.tcp(5432),
            description=f"tdx-prover {aws_region}: Allow inbound traffic on port 5432 from ECS tasks",  # noqa: E501
        )

        # Create a security group for the Lambda function
        lambda_security_group = ec2.SecurityGroup(
            self,
            f"{APP_SHORTNAME}LambdaSecurityGroup",
            vpc=vpc,
            description=f"Security group for {APP_SHORTNAME} Lambda function",
            allow_all_outbound=True,
            security_group_name=f"{APP_SHORTNAME}-lambda-sg",
        )

        # Add ingress rule to the database security group to allow access from Lambda
        db_security_group.add_ingress_rule(
            peer=ec2.Peer.security_group_id(lambda_security_group.security_group_id),
            connection=ec2.Port.tcp(5432),
            description=f"Allow PostgreSQL access from {APP_SHORTNAME} Lambda function",
        )

        # Create Rust Lambda Function for Tdx Prover
        # Note: This Lambda function expects the Rust binary to be built using cargo-lambda:
        # 1. Install cargo-lambda: cargo install cargo-lambda
        # 2. Build the Lambda binary: cargo lambda build --release
        # 3. The binary will be available in ../../target/lambda directory
        rust_lambda = _lambda.Function(
            self,
            f"{APP_SHORTNAME}-rust-lambda",
            function_name=f"{APP_SHORTNAME}-rust-lambda",
            runtime=_lambda.Runtime.PROVIDED_AL2,
            handler="bootstrap",
            code=_lambda.Code.from_asset("../target/lambda/tdx-prover/bootstrap.zip"),
            environment={
                "LAMBDA": "true",
                "DATABASE_URL": self.service_secrets.secret_value_from_json("DATABASE_URL").unsafe_unwrap(),
            },
            vpc=vpc,
            security_groups=[lambda_security_group],
        )

        # Grant the Lambda function permission to read the secrets
        self.service_secrets.grant_read(rust_lambda)

        # Reference the existing event bus from another stack
        event_bus = event_source.EventBus.from_event_bus_arn(
            self, 
            f"{APP_SHORTNAME}-existing-event-bus", 
            event_bus_arn=event_bus_arn
        )
        
        # Create a new rule that will forward events to our Lambda
        # This rule will coexist with the rule in the other stack
        rule = event_source.Rule(
            self,
            f"{APP_SHORTNAME}-event-rule",
            rule_name=f"{APP_SHORTNAME}-event-rule",
            event_bus=event_bus,
            description=f"Rule to forward events from {APP_SHORTNAME} event bus to Lambda",
            # Match the same events as the rule in the other stack
            event_pattern=event_source.EventPattern(
                source=["com.magic.newton"],
            ),
            targets=[event_targets.LambdaFunction(rust_lambda)]
        )

        # Allow EventBridge to invoke this Lambda function
        rust_lambda.add_permission(
            id=f"{APP_SHORTNAME}-eventbridge-invoke-permission",
            principal=iam.ServicePrincipal("events.amazonaws.com"),
            action="lambda:InvokeFunction",
            source_arn=rule.rule_arn
        )
        
        # Add iam permissions for rust lambda execution to newton-ops event bus
        rust_lambda.add_to_role_policy(
            iam.PolicyStatement(
                actions=["events:PutEvents"],
                resources=["*"],
            )
        )


