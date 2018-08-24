NAME=howto
SEMVER_VERSION=$(shell grep version Cargo.toml | awk -F"\"" '{print $$2}' | head -n 1)
REPO=pbzweihander
SHELL := /bin/bash

test:
	docker run --rm \
		-v cargo-cache:/root/.cargo \
		-v $$PWD:/volume \
		-w /volume \
		-it clux/muslrust \
		cargo test
	sudo chown $$USER:$$USER -R target

cargo-publish:
	if curl -sSL https://crates.io/api/v1/crates/$(NAME)/versions | jq -r ".versions | .[].num" | grep -q $(SEMVER_VERSION); then \
		echo "Tag $(SEMVER_VERSION) already exists - not publishing" ; \
	else \
		cargo publish ; \
	fi
