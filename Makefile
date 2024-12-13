POSTGRES_DIR="./crates/adapter/src/postgres"
DATABASE_URL="postgres://postgres:changeme@10.20.10.121:5432/postgrest"
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

setup-db:
	diesel setup --database-url ${DATABASE_URL} \
     --migration-dir ${POSTGRES_DIR}/migrations \
     --config-file ${POSTGRES_DIR}/diesel.toml

migrate:
	diesel migration run --database-url ${DATABASE_URL} \
     --migration-dir ${POSTGRES_DIR}/migrations \
     --config-file ${POSTGRES_DIR}/diesel.toml

migrate-redo:
	diesel migration redo --database-url ${DATABASE_URL} \
     --migration-dir ${POSTGRES_DIR}/migrations \
     --config-file ${POSTGRES_DIR}/diesel.toml
