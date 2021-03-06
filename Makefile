# Project variables.

REGISTRY ?= ghcr.io
USERNAME ?= cosi-spec
TAG ?= $(shell git describe --tag --always --dirty)
IMAGE := $(REGISTRY)/$(USERNAME)/engine:$(TAG)

# Build variables.

BUILD := docker buildx build
PLATFORM ?= linux/amd64
PROGRESS ?= auto
PUSH ?= false
ARG_BASE_IMAGE ?= ubuntu:20.04
ARG_RUST_VERSION ?= 1.50.0
COMMON_ARGS := --file=Dockerfile
COMMON_ARGS += --progress=$(PROGRESS)
COMMON_ARGS += --platform=$(PLATFORM)
COMMON_ARGS += --push=$(PUSH)
COMMON_ARGS += --build-arg=BASE_IMAGE=$(ARG_BASE_IMAGE)
COMMON_ARGS += --build-arg=RUST_VERSION=$(ARG_RUST_VERSION)

PROTO_FILE ?= https://raw.githubusercontent.com/andrewrynhard/specification/87d89fabe2be5cb818e652181749e418afe7a454/spec.proto

# Misc.

# N.B.: this list should be present in .gitignore.
ARTIFACTS := binaries target

all: lint test artifacts image ## Runs the complete set of targets.

lint: ## Lints the source code.
	$(BUILD) $(COMMON_ARGS) --target=$@ .

test: ## Runs the tests.
	$(BUILD) $(COMMON_ARGS) --target=$@ .

artifacts: ## Generates code and builds the binaries.
	$(BUILD) $(COMMON_ARGS) --output=type=local,dest=. --target=$@ .

image: ## Builds the image.
	$(BUILD) $(COMMON_ARGS) --output=type=image,name=$(IMAGE) --target=$@ .

define HELP_MENU_HEADER
\033[0;31mGetting Started\033[0m

To build this project, you must have the following installed:

- git
- make
- docker (19.03 or higher)
- buildx (https://github.com/docker/buildx)

The build process makes use of features not currently supported by the default
builder instance (docker driver). To create a compatible builder instance, run:

```
docker buildx create --driver docker-container --name local --use
```

If you already have a compatible builder instance, you may use that instead.

The artifacts (i.e. $(ARTIFACTS)) will be output to the root of the project.

endef

export HELP_MENU_HEADER

help: ## This help menu.
	@echo -e "$$HELP_MENU_HEADER"
	@grep -E '^[a-zA-Z%_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: proto
proto:
	curl \
		--silent \
		--show-error \
		--fail \
		--location \
		--remote-name \
		--output-dir ./proto \
		--create-dirs \
		$(PROTO_FILE)

run: artifacts ## Builds and runs the project.
	docker run --rm -it --privileged -v $(PWD)/binaries:/system -v $(PWD)/examples:/examples -p 50000:50000 --entrypoint=/system/engine --name cosi $(ARG_BASE_IMAGE)

clean: ## Removes the asset directory.
	-rm -rfv $(ARTIFACTS)
