#!/usr/bin/env python
from pyln.client import Plugin

import logging
import json

logger = logging.getLogger(__name__)

# This is a dummy plugin
# A node running this plugin (falsely) advertises itself to be an LSP
# It does not implement any LSP-functionality
plugin = Plugin(
    node_features="0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    dynamic=False,
)

def log_custom_msg(**kwargs):
    payload = kwargs.get("payload", "")
    peer_id = kwargs.get("peer_id", "") 

    plugin.log(f"Received CustomMsg from peer={peer_id} and payload={payload}")

    byte_arr = bytearray.fromhex(payload)
    json_rpc = json.loads(byte_arr[2:])

    plugin.log(f"BOLT8_MSG_ID={byte_arr[:2]} and payload={json_rpc}")

    return { "result" : "continue" }

if __name__ == "__main__":
    plugin.add_hook("custommsg", log_custom_msg)
    plugin.run()
