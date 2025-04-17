.PHONY: clean deploy synth install diff docs ci-deploy ecr-deploy

## Setting up quick commands for the CDK project
# when using "uv run" you need to also utilize "npx" so
# that the CDK package that is installed local to the project
# is used rather than any global installed version.

# synth the stack
synth:
	uv run npx cdk synth 

# deploy the stack
deploy:
	uv run npx cdk deploy 

# CI deploy
ci-deploy:
	uv run npx cdk deploy --all --require-approval never

# CI deploy
ecr-deploy:
	uv run npx cdk deploy

# clean the directories
clean:
	rm -rf .venv node_modules cdk.out

# diff the stack
diff:
	uv run npx cdk diff --no-change-set 

# generate the documentation
docs:
	uv run npx cdk doc

# install the dependencies using asdf
install:
	asdf install
	npm install
	uv sync
