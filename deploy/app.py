#!/usr/bin/env python3
import os

import aws_cdk as cdk
from aws_cdk import Tags
from dotenv import load_dotenv

from app.tdx_prover_stack import TdxProver

# Determine the .env file based on the AWS_DEPLOY_ENV environment variable
deploy_env = os.getenv("AWS_DEPLOY_ENV", "stagef")
dotenv_path = f".env.{deploy_env}" if deploy_env != "default" else ".env"

# print the dotenv_path
print(f"Loading environment variables from: {dotenv_path}")

# Load environment variables from a .env file if it exists
if os.path.exists(dotenv_path):
    load_dotenv(dotenv_path=dotenv_path)

# Define tags to tag all resources created in the stack
tags = {"app": "tdx-prover", "env": os.getenv("CDK_DEPLOY_ENV")}

app = cdk.App()

git_commit = app.node.try_get_context("git_commit")
long_git_commit = app.node.try_get_context("long_git_commit")

# Provide a default value if git_commit is None or not a string
if not isinstance(git_commit, str):
    git_commit = "head"
if not isinstance(long_git_commit, str):
    long_git_commit = "head"

stack = TdxProver(
    app,
    "TdxProverUSW2",
    deploy_env=os.getenv("CDK_DEPLOY_ENV"),
    aws_region="us-west-2",
    app_shortname="tdx-prover",
    git_commit=git_commit,
    long_git_commit=long_git_commit,
    db_security_group_id=os.getenv("USW2_DB_SECURITY_GROUP_ID"),
    vpc_id=os.getenv("USW2_VPC_ID"),
    env=cdk.Environment(account=os.getenv("AWS_ACCOUNT"), region="us-west-2"),
    event_bus_arn=os.getenv("OPS_EVENT_BUS_ARN"),
)

## We don't need a secondary region for the Prover stack
# ap_stack = TdxProver(
#     app,
#     "TdxProverAPNE2",
#     deploy_env=os.getenv("CDK_DEPLOY_ENV"),
#     aws_region="ap-northeast-2",
#     app_shortname="tdx-prover",
#     git_commit=git_commit,
#     long_git_commit=long_git_commit,
#     db_security_group_id=os.getenv("APNE2_DB_SECURITY_GROUP_ID"),
#     vpc_id=os.getenv("APNE2_VPC_ID"),
#     env=cdk.Environment(account=os.getenv("AWS_ACCOUNT"), region="ap-northeast-2"),
#     event_bus_arn=os.getenv("OPS_EVENT_BUS_ARN"),
# )

for key, value in tags.items():
    Tags.of(stack).add(key, value)

app.synth()
