from pyln.testing.fixtures import *
from pyln.client.lightning import Millisatoshi
from test.fixtures import (
    lsps_server,
    lsps_client,
    get_server_plugin_path,
    developer_options,
)
import logging

import json

logger = logging.getLogger(__name__)


def test_lsps1_disabled(node_factory, lsps_client):
    """Server provides correct info in lsps0.list_protocols"""

    logger.debug("Starting LSPS-server")
    server_plugin = get_server_plugin_path()
    logger.debug(f"server-plugin: {server_plugin}")
    lsps_server: LightningNode = node_factory.get_node(
        options={"plugin": server_plugin, **developer_options()}
    )

    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps0.list_protocols", params="{}"
    )

    assert response["result"] == {"protocols": [0]}

    # Check that lsps1.get_info is disabled
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.create_order", params="{}"
    )
    assert response["error"]["code"] == -32601
    # Check that lsps1.create_order is disabled
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.create_order", params="{}"
    )
    assert response["error"]["code"] == -32601
    # Check that lsps1.get_order is disabled
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.get_order", params="{}"
    )
    assert response["error"]["code"] == -32601


def test_lsps1_get_info(lsps_server, lsps_client):
    """Server responds correctly to lsps1.get_info"""
    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.get_info", params="{}"
    )

    # Returned an rpc response

    assert "result" in response, f"Error: {response}"
    result = response["result"]

    assert "options" in result


def test_lsps1_create_order_violate_options(lsps_server, lsps_client):
    # Set the config options here once they are dynamic
    # This would make the code more readable

    lsps_client.connect(lsps_server)

    params = dict(
        lsp_balance_sat="0",
        client_balance_sat="1000001",  # Too large
        funding_confirms_within_blocks=1,
        required_channel_confirmations=6,
        channel_expiry_blocks=144,
        announce_channel=False,
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    assert "error" in response, "Should be option mismatch but returned result"
    error = response["error"]

    assert error["code"] == 1000, str(error)
    assert error["message"] == "Option mismatch"
    assert error["data"]["property"] == "max_initial_client_balance_sat"


def test_lsps1_create_order(lsps_server, lsps_client):
    lsps_client.connect(lsps_server)

    params = dict(
        lsp_balance_sat="500000",
        client_balance_sat="0",
        funding_confirms_within_blocks=1,
        required_channel_confirmations=0,
        channel_expiry_blocks=144,
        announce_channel=False,
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    assert "result" in response, f"Error in response: {response}"
    result = response["result"]

    assert result["lsp_balance_sat"] == "500000"
    assert result["client_balance_sat"] == "0"
    assert result["funding_confirms_within_blocks"] == 1
    assert result["required_channel_confirmations"] == 0
    assert result["channel_expiry_blocks"] == 144
    assert result["announce_channel"] == False

    assert result["order_state"] == "CREATED"
    assert result["payment"]["state"] == "EXPECT_PAYMENT"


def test_lsps1_get_order_by_uuid(lsps_client, lsps_server):
    lsps_client.connect(lsps_server)

    params = dict(
        lsp_balance_sat="500000",
        client_balance_sat="0",
        funding_confirms_within_blocks=1,
        required_channel_confirmations=0,
        channel_expiry_blocks=144,
        announce_channel=False,
    
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    order_id = response["result"]["order_id"]
    params = json.dumps({"order_id": order_id})

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.get_order", params=params
    )

    assert "result" in response, f"Error in response: {response}"
    result = response["result"]

    assert result["lsp_balance_sat"] == "500000"
    assert result["client_balance_sat"] == "0"
    assert result["funding_confirms_within_blocks"] == 1
    assert result["required_channel_confirmations"] == 0
    assert result["channel_expiry_blocks"] == 144
    assert not result["announce_channel"]

    assert result["order_state"] == "CREATED"
    assert result["payment"]["state"] == "EXPECT_PAYMENT"


def test_pay_lsps1_order(lsps_client, lsps_server):
    # Connect the client to server and open an initial channel
    logger.info("Connecting and opening a channel")
    lsps_client.connect(lsps_server)
    lsps_client.openchannel(lsps_server)

    # Provide the lsp-server with 10 BTC so they can open a channel
    lsps_server.fundwallet(100_000_000 * 10)

    # Client requests a channel of 500_000 sats to the server
    logger.info("lsps1.create_order")
    params = dict(
        lsp_balance_sat="123456",
        client_balance_sat="0",
        funding_confirms_within_blocks=1,
        required_channel_confirmations=0,
        channel_expiry_blocks=144,
        announce_channel=False,
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    assert "result" in response, f"Error: {response}"

    order_id = response["result"]["order_id"]
    bolt11_invoice = response["result"]["payment"]["bolt11_invoice"]

    # Client pays for the invoice using lightning
    logger.info("Pay order using bolt11_invoice")
    pay_result = lsps_client.rpc.pay(bolt11_invoice)
    logger.info(f"{pay_result}")

    # Assert that core lightning has actually paid the invoice
    list_pay_result = lsps_client.rpc.listpays(bolt11_invoice)
    assert list_pay_result["pays"][0]["status"] == "complete"

    # Client does lsps1.get_order
    logger.info("retrieve lsps1.get_order")
    params = dict(order_id=order_id)
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.get_order",
        params=json.dumps(params),
    )

    assert "result" in response, f"Error in response: {response}"
    # Check if the order is considered paid
    assert response["result"]["payment"]["state"] == "PAID"
    assert response["result"]["order_state"] == "COMPLETED"

    peer_channels = lsps_client.rpc.listpeerchannels(lsps_server.info["id"])
    lsps_outpoint = response["result"]["channel"]["funding_outpoint"]

    # Find the cnannel with the matching outpoint
    channel = None
    for c in peer_channels["channels"]:
        c_outnum = c["funding_outnum"]
        c_funding_txid = c["funding_txid"]
        c_outpoint = f"{c_funding_txid}:{c_outnum}"
        if c_outpoint == lsps_outpoint:
            channel = c
            break

    if channel is None:
        assert (
            False
        ), "Failed to find channel with matching outpoint in listpeerchannels"

    assert channel["private"], "The channel should not be announced"
    assert channel["total_msat"] == Millisatoshi(123456000)
    assert channel["to_us_msat"] == Millisatoshi(0)


def test_server_complains_on_unrecognized_argument(lsps_server, lsps_client):
    """Server responds with Invalid Params and list unrecognized arguments"""
    lsps_client.connect(lsps_server)

    methods = ["lsps1.get_info", "lsps1.create_order", "lsps1.get_order"]

    for method in methods:
        logger.info("Checking method %s", method)
        response = lsps_client.rpc.lsps0_send_request(
            peer_id=lsps_server.info["id"],
            method=method,
            params=json.dumps({"param_a": "a"}),
        )

        assert response["error"]["data"]["unrecognized"] == ["param_a"]


def test_create_order_detects_invalid_param(lsps_server, lsps_client):
    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps({"lsp_balance_sat": 100}),
    )

    assert response["error"]["data"]["property"] == "lsp_balance_sat"
