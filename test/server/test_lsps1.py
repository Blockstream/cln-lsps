from pyln.testing.fixtures import *
from test.fixtures import lsps_server, lsps_client


def test_server_responds_to_lsps1_get_info(lsps_server, lsps_client):
    """Server responds correctly to lsps1.get_info"""
    lsps_client.connect(lsps_server)

    response = lsps_client.rpc.lsps0_send_request(
        peer_id=lsps_server.info["id"], method="lsps1.info", params="{}"
    )

    # Returned an rpc response
    result = response["result"]

    assert "options" in result
    assert "website" in result
