.DEFAULT_GOAL := all

all: lsps-server lsps-client

base:
	mkdir -p build/plugins

# Initiates a database that is used for testing
db: plugins/lsps-server/data/lsp_server.db

plugins/lsps-server/data/lsp_server.db: 
	mkdir -p plugins/lsps-server/data
	sqlx database create --database-url sqlite:plugins/lsps-server/data/lsp_server.db
	sqlx migrate run --database-url sqlite:plugins/lsps-server/data/lsp_server.db --source plugins/lsps-server/migrations

lsps-server: base plugins/lsps-server/data/lsp_server.db
	mkdir -p build/plugins/lsps-server
	cargo build -p lsps-server
	cp ./target/debug/lsps-server ./build/plugins/lsps-server/lsps-server

lsps-client: base
	mkdir -p build/plugins/lsps-client
	cargo build -p lsps-client
	cp ./target/debug/lsps-client ./build/plugins/lsps-client/lsps-client

lsps-client-test: lsps-client
	SET LSPS_CLIENT_PATH = $(CWD)/build/plugins
	python -m pytest tests

clean:
	cargo clean
	rm -rf ./build

