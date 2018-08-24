LIB_DIR=howto
BIN_DIR=howto-cli

all: test build

test:
	$(MAKE) -C $(LIB_DIR) test

build:
	$(MAKE) -C $(BIN_DIR) build

docker-push:
	$(MAKE) -C $(BIN_DIR) tag-semver
	$(MAKE) -C $(BIN_DIR) tag-latest

lib-publish:
	$(MAKE) -C $(LIB_DIR) cargo-publish

bin-publish:
	$(MAKE) -C $(BIN_DIR) cargo-publish
