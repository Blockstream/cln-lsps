import pytest
from pyln.testing.fixtures import *
from pyln.testing.utils import NodeFactory, LightningNode

import logging


logger = logging.getLogger(__name__)


def test_pyln_is_installed(node_factory: NodeFactory, bitcoind):
    n1: LightningNode = node_factory.get_node()
    n2: LightningNode = node_factory.get_node()

    n1.connect(n2)
    peers = n1.rpc.listpeers()

    n2_id = n2.info["id"]

    assert n2_id in [peer["id"] for peer in peers["peers"]]
