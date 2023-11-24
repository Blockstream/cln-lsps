import logging
import os

from threading import Thread

import typing as t

from pyln.testing.fixtures import *
from pyln.testing.utils import NodeFactory, LightningNode, wait_for

from test.util.options import lsps1_server_options

logger = logging.getLogger(__name__)


def get_plugin_dir_lsps0_dummy_server() -> str:
    cwd = os.getcwd()
    return os.environ.get(
        "LSPS0_CLIENT_PATH", os.path.join(cwd, "build/plugins/lsps0-server")
    )


def get_plugin_dir_lsps0_client() -> str:
    cwd = os.getcwd()
    return os.environ.get(
        "LSPS0_CLIENT_PATH", os.path.join(cwd, "build/plugins/lsps0-client")
    )


@pytest.mark.skip(reason="Takes too long and can time-out")
def test_lsps0_client_can_find_servers(node_factory: NodeFactory):
    """
    LSP-servers advertise themself by setting node-feature flag 729.
    New clients joining the network look for nodes setting that feature flag
    """
    # Load the lsps0-dummy-server plugin
    plugin_dummy_server = get_plugin_dir_lsps0_dummy_server()
    plugin_lsps0_client = get_plugin_dir_lsps0_client()

    logger.info("server plugin-dir: %s", plugin_dummy_server)
    logger.info("client plugin-dir: %s", plugin_lsps0_client)

    # Create a channel-graph
    # lsp1 and lsp2 run a plug-in which sets a feature flag that tells the lsp_client they are LSP-servers
    logger.info("Start lightning nodes")
    router_a: LightningNode = node_factory.get_node()
    lsp1: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_dummy_server}
    )
    lsp2: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_dummy_server}
    )

    # Creating a channel-graph and wait for gossip to be completed
    # Waiting for gossip is slow. You might want to set SLOW_MACHINE=1
    # to ensure the test doesn't time-out
    logger.info("Creating the channel-graph")
    node_factory.join_nodes(nodes=[router_a, lsp1, lsp2], wait_for_announce=True)

    # Create the lsp_client.
    # We need to connect to a node to receive all gossip
    logger.info("Start LSP-client lightning node")
    lsp_client: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_lsps0_client}
    )
    lsp_client.connect(router_a)
    lsp_client.connect(lsp1)
    lsp_client.connect(lsp2)

    # Wait until the lsp_client is aware of all the recent gossip
    logger.info("Waiting for gossip")
    wait_for(lambda: len(lsp_client.rpc.listnodes()["nodes"]) == 3)

    # Let's query for all existing LSP-servers
    logger.info("Use the LSP-client rpc command")
    result = lsp_client.rpc.lsps0_list_servers()

    assert lsp1.info["id"] in result
    assert lsp2.info["id"] in result

    assert not router_a.info["id"] in result


def test_lsps0_list_protocols(node_factory: NodeFactory):
    """
    Calls lsps0.list_protocols to see which LSPS a sever supports
    """
    # Load the lsps0-dummy-server plugin
    plugin_dummy_server = get_plugin_dir_lsps0_dummy_server()
    plugin_lsps0_client = get_plugin_dir_lsps0_client()

    logger.info("server plugin-dir: %s", plugin_dummy_server)
    logger.info("client plugin-dir: %s", plugin_lsps0_client)

    # Starting the client and server
    logger.info("Launching Lightning nodes")
    lsps_server: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_dummy_server}
    )
    lsps_client: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_lsps0_client}
    )

    # Ensure the lsp_client and lsp_server are connected
    lsps_client.connect(lsps_server)

    # Extract the node_id's of the server and the client
    server_node_id = lsps_server.info["id"]
    client_node_id = lsps_client.info["id"]
    logger.info("LSPS-server with node_id=%s", server_node_id)
    logger.info("LSPS-client with node_id=%s", client_node_id)

    logger.info("Client requests lsps0.list_protocols")
    result = lsps_client.rpc.lsps0_list_protocols(server_node_id)
    protocols = result["protocols"]

    assert len(protocols) >= 0
    assert 0 in protocols
    assert 1 in protocols


def test_lsps0_get_info(node_factory: NodeFactory):
    """
    Calls lsps0.get_info to get pricing information
    """
    # Load the lsps0-dummy-server plugin
    plugin_dummy_server = get_plugin_dir_lsps0_dummy_server()
    plugin_lsps0_client = get_plugin_dir_lsps0_client()

    logger.info("server plugin-dir: %s", plugin_dummy_server)
    logger.info("client plugin-dir: %s", plugin_lsps0_client)

    # Starting the client and server
    logger.info("Launching Lightning nodes")
    lsp_server_options = lsps1_server_options()
    lsps_server: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_dummy_server, **lsp_server_options}
    )
    lsps_client: LightningNode = node_factory.get_node(
        options={"plugin-dir": plugin_lsps0_client}
    )

    # Ensure the lsp_client and lsp_server are connected
    lsps_client.connect(lsps_server)

    # Extract the node_id's of the server and the client
    server_node_id = lsps_server.info["id"]
    client_node_id = lsps_client.info["id"]
    logger.info("LSPS-server with node_id=%s", server_node_id)
    logger.info("LSPS-client with node_id=%s", client_node_id)

    logger.info("Client requests lsps1.list_protocols")
    result = lsps_client.rpc.lsps1_get_info(server_node_id)

    assert result["supported_versions"] == [
        1
    ], "The respone should include supported_versions"
    website = result["website"]
    assert website is None
    assert "options" in result, "The response should have an options dict"
