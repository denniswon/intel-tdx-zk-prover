import aws_cdk as cdk
from aws_cdk import CfnOutput
from aws_cdk import Duration
from aws_cdk import Stack
from aws_cdk import aws_certificatemanager as acm
from aws_cdk import aws_ec2 as ec2
from aws_cdk import aws_ecr as ecr
from aws_cdk import aws_ecs as ecs
from aws_cdk import aws_elasticache as elasticache
from aws_cdk import aws_elasticloadbalancingv2 as elbv2
from aws_cdk import aws_iam as iam
from aws_cdk import aws_logs as logs
from aws_cdk import aws_secretsmanager as secretsmanager
from constructs import Construct


class TdxProver(Stack):
    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        deploy_env: str,
        app_shortname: str,
        db_security_group_id: str,
        ecs_cluster: str,
        git_commit: str,
        long_git_commit: str,
        vpc_id: str,
        aws_region: str,
        ecr_repository_arn: str,
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

        if aws_region == PRIMARY_REGION:
            # Get the GitHub OIDC provider
            github_oidc_provider = iam.OpenIdConnectProvider.from_open_id_connect_provider_arn(
                self,
                "GitHubOIDCProvider",
                open_id_connect_provider_arn=f"arn:aws:iam::{self.account}:oidc-provider/token.actions.githubusercontent.com",
            )

            # Create role for tdx-prover repository
            role = iam.Role(
                self,
                "GithubRole-tdx-prover",
                role_name="github-magiclabs-tdx-prover-role",
                assumed_by=iam.WebIdentityPrincipal(
                    github_oidc_provider.open_id_connect_provider_arn,
                    conditions={
                        "StringEquals": {
                            "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
                        },
                        "StringLike": {
                            "token.actions.githubusercontent.com:sub": "repo:magiclabs/tdx-prover:*"
                        },
                    },
                ),
            )

            # Create a managed policy with the desired name
            cdk_deploy_policy = iam.ManagedPolicy(
                self,
                "CDKDeployPolicy",
                managed_policy_name="CDKDeployPolicy",
                statements=[
                    iam.PolicyStatement(
                        effect=iam.Effect.ALLOW,
                        actions=["sts:AssumeRole"],
                        resources=[
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-deploy-role-{self.account}-us-west-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-file-publishing-role-{self.account}-us-west-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-image-publishing-role-{self.account}-us-west-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-lookup-role-{self.account}-us-west-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-deploy-role-{self.account}-ap-northeast-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-file-publishing-role-{self.account}-ap-northeast-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-image-publishing-role-{self.account}-ap-northeast-2",
                            f"arn:aws:iam::{self.account}:role/cdk-hnb659fds-lookup-role-{self.account}-ap-northeast-2",
                        ],
                    )
                ],
            )

            role.add_managed_policy(cdk_deploy_policy)

        vpc = ec2.Vpc.from_lookup(self, "ExistingVPC", vpc_id=vpc_id)

        # Use the passport-identity ecs clusters
        cluster = ecs.Cluster.from_cluster_attributes(
            self, "ExistingCluster", cluster_name=ecs_cluster, vpc=vpc
        )

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
            port=8000,
            protocol=elbv2.ApplicationProtocol.HTTP,
            target_type=elbv2.TargetType.IP,
            health_check=elbv2.HealthCheck(
                path="/health",
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
                secret_string_template='{"DD_API_KEY": ""}',
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

        # Create a task definition
        task_definition = ecs.FargateTaskDefinition(
            self,
            f"{APP_SHORTNAME}-td",
            cpu=2048,
            memory_limit_mib=4096,
        )

        # Grant the task role permission to read the secret
        task_definition.task_role.add_to_policy(
            iam.PolicyStatement(
                actions=[
                    "secretsmanager:GetSecretValue",
                ],
                resources=["*"],
            )
        )

        # Additionally add the SSM permissions for session manager to allow EXEC into running containers
        task_definition.task_role.add_to_policy(
            iam.PolicyStatement(
                actions=[
                    "ssmmessages:CreateControlChannel",
                    "ssmmessages:CreateDataChannel",
                    "ssmmessages:OpenControlChannel",
                    "ssmmessages:OpenDataChannel",
                    "secretsmanager:BatchGetSecretValue",
                    "secretsmanager:GetSecretValue",
                    "secretsmanager:ListSecrets",
                ],
                resources=["*"],
            )
        )

        # Fetch the Datadog API key from Secrets Manager using ARN
        datadog_api_key = self.service_secrets.secret_value_from_json("DD_API_KEY").unsafe_unwrap()

        # Create a container
        container = task_definition.add_container(
            f"{APP_SHORTNAME}-container",
            image=ecs.ContainerImage.from_ecr_repository(ecr_repo, tag=git_commit),
            container_name=f"{APP_SHORTNAME}-container",
            cpu=1536,
            memory_limit_mib=3072,
            environment={
                "RUNTIME": "docker",
                "DEPLOY_ENV": f"{deploy_env}",
                "DD_APM_ENABLED": "true",
                "DD_SERVICE": f"{APP_SHORTNAME}",
                "DD_ENV": f"{deploy_env}",
                "DD_LOGS_INJECTION": "true",
                "DD_LOGS_ENABLED": "true",
                "DD_LOGS_CONFIG_CONTAINER_COLLECT_ALL": "true",
                "DD_GIT_REPOSITORY_URL": "https://github.com/magiclabs/tdx-prover",
                "DD_GIT_COMMIT_SHA": long_git_commit,
                "DD_VERSION": git_commit,
                "TDXPROVER_ENV": f"{deploy_env}",
            },
            secrets={
                "DD_API_KEY": ecs.Secret.from_secrets_manager(self.service_secrets, "DD_API_KEY"),
                "DATABASE": ecs.Secret.from_secrets_manager(self.service_secrets, "DATABASE"),
            },
            # Firelens logging
            logging=ecs.LogDrivers.firelens(
                options={
                    "Name": "datadog",
                    "apikey": datadog_api_key,
                    "dd_service": f"{APP_SHORTNAME}",
                    "dd_source": "ecs",
                    "dd_tags": f"env:{deploy_env},service:{APP_SHORTNAME}",
                    "TLS": "on",
                    "provider": "ecs",
                }
            ),
        )

        container.add_port_mappings(ecs.PortMapping(container_port=8002))

        # Add Firelens log router container with custom resource configuration
        task_definition.add_firelens_log_router(
            "log-router",
            image=ecs.ContainerImage.from_registry("amazon/aws-for-fluent-bit:stable"),
            firelens_config=ecs.FirelensConfig(
                type=ecs.FirelensLogRouterType.FLUENTBIT,
                options=ecs.FirelensOptions(
                    config_file_type=ecs.FirelensConfigFileType.FILE,
                    config_file_value="/fluent-bit/configs/parse-json.conf",
                ),
            ),
            memory_reservation_mib=256,  # Allocate more memory to the log router
            cpu=128,  # Allocate more CPU to the log router
            logging=ecs.LogDrivers.aws_logs(stream_prefix="log-router"),
        )

        #######################################################################
        # Add DataDog container for Trace and APM
        #######################################################################

        datadog_container = task_definition.add_container(
            "DatadogAgent",
            container_name=f"{APP_SHORTNAME}-datadog-agent",
            image=ecs.ContainerImage.from_registry("public.ecr.aws/datadog/agent:latest"),
            essential=False,
            environment={
                "DD_SITE": "datadoghq.com",
                "ECS_FARGATE": "true",
                "DD_PROCESS_AGENT_ENABLED": "true",
                "DD_TAGS": f"project:{APP_SHORTNAME}",
                "DD_CONTAINER_EXCLUDE": "name:datadog-agent",
                "DD_DOGSTATSD_NON_LOCAL_TRAFFIC": "true",
                "DD_APM_NON_LOCAL_TRAFFIC": "true",
                "DEPLOY_ENV": f"{deploy_env}",
                "DD_APM_ENABLED": "true",
                "DD_SERVICE": f"{APP_SHORTNAME}",
                "DD_ENV": f"{deploy_env}",
                "DD_LOGS_INJECTION": "true",
                "DD_LOGS_ENABLED": "true",
                "DD_LOGS_CONFIG_CONTAINER_COLLECT_ALL": "true",
                "DD_GIT_REPOSITORY_URL": "https://github.com/magiclabs/tdx-prover",
                "DD_GIT_COMMIT_SHA": long_git_commit,
                "DD_VERSION": git_commit,
            },
            secrets={
                "DD_API_KEY": ecs.Secret.from_secrets_manager(self.service_secrets, "DD_API_KEY"),
            },
            logging=ecs.AwsLogDriver(
                stream_prefix="datadog", log_retention=logs.RetentionDays.ONE_WEEK
            ),
            stop_timeout=Duration.seconds(5),
        )

        datadog_container.add_port_mappings(
            ecs.PortMapping(container_port=8126, host_port=8126, protocol=ecs.Protocol.TCP),
            ecs.PortMapping(container_port=8125, host_port=8125, protocol=ecs.Protocol.UDP),
        )

        #######################################################################
        ## Service Initialization                                             ##
        #######################################################################

        # Create a service for the api container
        api_service = ecs.FargateService(
            self,
            f"{APP_SHORTNAME}-service",
            service_name=f"{APP_SHORTNAME}-service",
            cluster=cluster,
            task_definition=task_definition,
            security_groups=[task_security_group],
            desired_count=0,
            deployment_controller=ecs.DeploymentController(type=ecs.DeploymentControllerType.ECS),
            min_healthy_percent=50,
            max_healthy_percent=200,
            enable_execute_command=True,
        )

        # Attach the service to the target group
        api_service.attach_to_application_target_group(target_group)

        ###### Output from CDK ######

        CfnOutput(
            self,
            "TaskDefinitionVersion",
            value=f"{task_definition.family}:{task_definition.task_definition_arn.split(':')[-1]}",
            description=f"{APP_SHORTNAME} Task Definition Version",
        )

        CfnOutput(
            self,
            "LoadBalancerDNS",
            value=lb.load_balancer_dns_name,
            description="LB DNS",
        )

