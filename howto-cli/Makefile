NAME=howto
CRATE_NAME=howto-cli
BIN_NAME=howto
VERSION=$(shell git rev-parse HEAD)
SEMVER_VERSION=$(shell grep version Cargo.toml | awk -F"\"" '{print $$2}' | head -n 1)
REPO=pbzweihander
SHELL := /bin/bash

build:
	docker run --rm \
		-v cargo-cache:/root/.cargo \
		-v $$PWD:/volume \
		-w /volume \
		-it clux/muslrust \
		cargo build --release
	sudo chown $$USER:$$USER -R target
	strip target/x86_64-unknown-linux-musl/release/$(BIN_NAME)
	mv target/x86_64-unknown-linux-musl/release/$(BIN_NAME) .

docker-build:
	@echo "Reusing built binary in current directory from make compile"
	@ls -lah ./$(BIN_NAME)
	docker build -t $(REPO)/$(NAME):$(VERSION) .

tag-latest: docker-build
	docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):latest
	docker push $(REPO)/$(NAME):latest

tag-semver: docker-build
	if curl -sSL https://registry.hub.docker.com/v1/repositories/$(REPO)/$(NAME)/tags | jq -r ".[].name" | grep -q $(SEMVER_VERSION); then \
		echo "Tag $(SEMVER_VERSION) already exists - not publishing" ; \
	else \
		docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):$(SEMVER_VERSION) ; \
		docker push $(REPO)/$(NAME):$(SEMVER_VERSION) ; \
	fi

cargo-publish:
	if curl -sSL https://crates.io/api/v1/crates/$(CRATE_NAME)/versions | jq -r ".versions | .[].num" | grep -q $(SEMVER_VERSION); then \
		echo "Tag $(SEMVER_VERSION) already exists - not publishing" ; \
	else \
		cargo publish ; \
	fi
