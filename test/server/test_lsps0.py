import logging
from pyln.testing.fixtures import *
from test.fixtures import lsps_server, lsps_client

logger = logging.getLogger(__name__)


def test_server_responds_to_unknown_method(lsps_server, lsps_client):
    """Server responds correctly to unknown method"""
    # Ensure the lsp_client and lsp_server are connected
    logger.info("Connect to the server")
    lsps_client.connect(lsps_server)

    lsps_id = lsps_server.info["id"]

    # Send a request
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_id, method="lsps0.method_does_not_exist", params="{}"
    )

    assert response["error"]["code"] == -32601
    assert response["error"]["message"] == "Method not found"


def test_server_responds_to_lsps0_list_protocols(lsps_server, lsps_client):
    """Server responds correctly to list-protocols"""
    lsps_client.connect(lsps_server)
    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps0.list_protocols", params="{}"
    )

    assert response["result"]["protocols"] == [0, 1]


def test_server_complains_on_unrecognized_argument(lsps_server, lsps_client):
    """Server responds with Invalid Params and list unrecognized arguments"""
    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"],
        method="lsps0.list_protocols",
        params=json.dumps({"param_a": "a"}),
    )

    assert response["error"]["data"]["unrecognized"] == ["param_a"]
