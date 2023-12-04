.DEFAULT_GOAL := all

all: lsps0-server lsps0-client

base:
	mkdir -p build/plugins

plugins/lsps0-server/data/lsp_server.db: 
	mkdir -p plugins/lsps0-server/data
	sqlx database create --database-url sqlite:plugins/lsps0-server/data/lsp_server.db
	sqlx migrate run --database-url sqlite:plugins/lsps0-server/data/lsp_server.db --source plugins/lsps0-server/migrations

lsps0-server: base plugins/lsps0-server/data/lsp_server.db
	mkdir -p build/plugins/lsps0-server
	cargo build -p lsps0-server
	cp ./target/debug/lsps0-server ./build/plugins/lsps0-server/lsps0-server

lsps0-client: base
	mkdir -p build/plugins/lsps0-client
	cargo build -p lsps0-client
	cp ./target/debug/lsps0-client ./build/plugins/lsps0-client/lsps0-client

lsps0-client-test: lsps0-client
	SET LSPS0_CLIENT_PATH = $(CWD)/build/plugins
	python -m pytest tests

clean:
	cargo clean
	rm -rf ./build

