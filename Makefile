PKG_NAME=zksteve/frog
BUILD_VERSION=$(shell git describe --long)
BUILD_RELEASE=$(shell git describe --tags --abbrev=0)

cargo-fmt:
	taplo fmt --config taplo/taplo.toml

lint:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy --fix --allow-dirty --allow-staged

build:
	export BUILDKIT_PROGRESS=plain && \
	export DOCKER_BUILDKIT=1 && \
	docker build -t $(PKG_NAME)-server:$(BUILD_VERSION) --target=server . && \
	docker build -t $(PKG_NAME)-client:$(BUILD_VERSION) --target=client . && \
	docker build -t $(PKG_NAME)-worker:$(BUILD_VERSION) --target=worker .

push:
	docker push $(PKG_NAME)-server:$(BUILD_VERSION) && \
	docker push $(PKG_NAME)-client:$(BUILD_VERSION) && \
	docker push $(PKG_NAME)-worker:$(BUILD_VERSION)
