import aws_cdk as cdk
from aws_cdk import CfnOutput
from aws_cdk import Stack
from aws_cdk import aws_ecr as ecr
from constructs import Construct


class TdxProverEcrStack(Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # Add tags to all resources in the stack
        cdk.Tags.of(self).add("project", "tdx-prover")
        cdk.Tags.of(self).add("stack", "ecr-repository")

        # Create the ECR repository
        ecr_repo = ecr.Repository(
            self,
            "tdxProverRepo",
            repository_name="tdx-prover",
            lifecycle_rules=[
                ecr.LifecycleRule(
                    rule_priority=1,
                    description="Limit to 50 tagged images",
                    max_image_count=50,
                    tag_status=ecr.TagStatus.ANY,
                )
            ],
        )

        CfnOutput(
            self,
            "EcrRepositoryArn",
            value=ecr_repo.repository_arn,
            description="The ARN of the ECR repository",
        )
