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

        # Create a security group for the database
        db_security_group = ec2.SecurityGroup(
            self,
            f"TdxProverDbSecurityGroup-{aws_region}",
            vpc=vpc,
            description="Allow inbound traffic on port 5432 from ECS tasks",
            allow_all_outbound=True,
            security_group_name=f"tdx-prover-{aws_region}-db-sg",
        )

        # Look up existing db security
        existing_db_security_group = ec2.SecurityGroup.from_security_group_id(
            self,
            "ExistingDbSecurityGroup",
            db_security_group_id,
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

        # Add an ingress rule to the existing security group
        existing_db_security_group.add_ingress_rule(
            peer=ec2.Peer.security_group_id(lambda_security_group.security_group_id),
            connection=ec2.Port.tcp(5432),
            description=f"Allow PostgreSQL access from {APP_SHORTNAME} Lambda function",
        )

        # Create a CloudWatch logs group for the Lambda function
        log_group = logs.LogGroup(
            self,
            f"{APP_SHORTNAME}-lambda-logs",
            log_group_name=f"/aws/lambda/{APP_SHORTNAME}-rust-lambda",
            retention=logs.RetentionDays.ONE_MONTH,
            removal_policy=cdk.RemovalPolicy.DESTROY,
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
            runtime=_lambda.Runtime.PROVIDED_AL2023,
            handler="bootstrap",
            code=_lambda.Code.from_asset("../target/lambda/tdx-prover/bootstrap.zip"),
            environment={
                "LAMBDA": "true",
                "DATABASE_URL": self.service_secrets.secret_value_from_json("DATABASE_URL").unsafe_unwrap(),
                "RUST_BACKTRACE": "1",  # Enable backtraces for better debugging
                "DEFAULT_RPC_URL": "https://mainnet.base.org",
                "DEFAULT_DCAP_CONTRACT": "0x9E4a45c40e06CE0653C33769138dF48802c1CF1e",
                "ENCLAVE_ID_DAO_ADDRESS": "0xd74e880029cd3b6b434f16bea5f53a06989458ee",
                "FMSPC_TCB_DAO_ADDRESS": "0xd3a3f34e8615065704ccb5c304c0ced41bb81483",
                "PCS_DAO_ADDRESS": "0xb270cd8550da117e3accec36a90c4b0b48dad342",
                "PCK_DAO_ADDRESS": "0xa4615c2a260413878241ff7605ad9577feb356a5",
                "VERIFY_ONLY": "false",
                "SP1_PROVER": "network",
                "NETWORK_PRIVATE_KEY": self.service_secrets.secret_value_from_json("NETWORK_PRIVATE_KEY").unsafe_unwrap(),
                "PROVER_PRIVATE_KEY": self.service_secrets.secret_value_from_json("PROVER_PRIVATE_KEY").unsafe_unwrap(),
                "SQLX_OFFLINE": "false",
                "ENV": "prod",
                "RUST_LOG": "debug",
            },
            vpc=vpc,
            security_groups=[lambda_security_group],
            log_group=log_group,
            memory_size=1024,  # Increased memory for better performance
            timeout=Duration.seconds(300),  # Increased timeout to 5 minutes for processing
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



