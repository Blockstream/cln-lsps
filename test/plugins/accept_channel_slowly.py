#!/usr/bin/env python
"""
This plugin is mainly used to test and demo LSPS1.

This plugin can
- reject incoming channels
- delay responding to incoming channels

This functionality is useful for testing the server implementation
of HODL invoices.

It helps us to assert that
- payments are refunded if the peer rejects the channel
- channel_opens are aborted and payments are refunded 
  if the channel is not accepted in a reasonable time-frame
"""
from pyln.client import Plugin

plugin = Plugin(dynamic=True)

plugin.add_option(
    name="test-wait-incoming-channel",
    default=0,
    description="The amount of seconds to wait when accepting a new channel",
    opt_type="int",
)

plugin.add_option(
    name="test-reject-incoming-channel",
    description="When this flag is set any channel open request will be rejected",
    opt_type="flag",
    default=False,
)


@plugin.hook("openchannel")
def openchannel(**kwargs):
    must_reject = plugin.get_option("test-reject-incoming-channel")
    if must_reject:
        return {"result": "reject"}

    sleep_time = plugin.get_option("test-wait-incoming-channel")
    sleep_time = int(sleep_time)
    plugin.log(f"Sleep {sleep_time} seconds before accepting the channel")
    return {"result": "continue"}


if __name__ == "__main__":
    plugin.run()
