import logging
import os

from pyln.testing.fixtures import node_factory
from pyln.testing.utils import NodeFactory, LightningNode

import pytest

from test.util.options import lsps1_server_options, developer_options

logger = logging.getLogger(__name__)


def get_server_plugin_path() -> str:
    cwd = os.getcwd()
    return os.environ.get(
        "LSPS_SERVER_PLUGIN_PATH",
        os.path.join(cwd, "build/plugins/lsps0-server/lsps0-server"),
    )


def get_client_plugin_path() -> str:
    cwd = os.getcwd()
    return os.environ.get(
        "LSPS_CLIENT_PLUGIN_PATH",
        os.path.join(cwd, "build/plugins/lsps0-client/lsps0-client"),
    )


@pytest.fixture
def lsps_server(node_factory: NodeFactory) -> LightningNode:
    logger.debug("Starting LSPS-server")
    server_plugin = get_server_plugin_path()
    logger.debug(f"server-plugin: {server_plugin}")
    lsps_server: LightningNode = node_factory.get_node(
        options={"plugin": server_plugin, **lsps1_server_options(), **developer_options()}
    )
    return lsps_server


@pytest.fixture
def lsps_client(node_factory: NodeFactory) -> LightningNode:
    logger.debug("Starting LSPS-client")
    client_plugin = get_client_plugin_path()
    logger.debug(f"client-plugin: {client_plugin}")
    lsps_client = node_factory.get_node(options={"plugin": client_plugin, **developer_options()})
    return lsps_client
