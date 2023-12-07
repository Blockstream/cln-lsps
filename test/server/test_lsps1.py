from pyln.testing.fixtures import *
from test.fixtures import lsps_server, lsps_client
import logging

logger = logging.getLogger(__name__)


def test_lsps1_get_info(lsps_server, lsps_client):
    """Server responds correctly to lsps1.get_info"""
    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.info", params="{}"
    )

    # Returned an rpc response
    result = response["result"]

    assert "options" in result
    assert "website" in result


def test_lsps1_create_order_violate_options(lsps_server, lsps_client):
    # Set the config options here once they are dynamic
    # This would make the code more readable

    lsps_client.connect(lsps_server)

    params = dict(
        api_version=1,
        lsp_balance_sat="0",
        client_balance_sat="1000001",  # Too large
        confirms_within_blocks=1,
        channel_expiry_blocks=144,
        announceChannel=False,
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    assert "error" in response
    error = response["error"]

    assert error["code"] == 1000
    assert error["message"] == "Option mismatch"
    assert error["data"]["property"] == "max_initial_client_balance_sat"


def test_lsps1_create_order(lsps_server, lsps_client):
    lsps_client.connect(lsps_server)

    params = dict(
        api_version=1,
        lsp_balance_sat="500000",
        client_balance_sat="0",
        confirms_within_blocks=1,
        channel_expiry_blocks=144,
        announceChannel=False,
    )

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps1.create_order",
        params=json.dumps(params),
    )

    result = response["result"]

    assert result["lsp_balance_sat"] == "500000"
    assert result["client_balance_sat"] == "0"
    assert result["confirms_within_blocks"] == 1
    assert result["channel_expiry_blocks"] == 144
    assert result["announceChannel"] == False

    assert result["order_state"] == "CREATED"
    assert result["payment"]["state"] == "EXPECT_PAYMENT"
