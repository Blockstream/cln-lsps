import typing as t


def developer_options() -> t.Dict[str, t.Any]:
    return {
        "allow-deprecated-apis" : False,
        "developer" : None,
        "dev-fast-gossip" : None,
        "dev-bitcoind-poll" : 5
    }

def lsps1_server_options() -> t.Dict[str, t.Any]:
    return {
        "lsps1-enable": True,
        "lsps1-min-channel-balance-sat": "0",
        "lsps1-max-channel-balance-sat": "1000000",
        "lsps1-min-initial-client-balance-sat": "0",
        "lsps1-max-initial-client-balance-sat": "0",
        "lsps1-min-initial-lsp-balance-sat": "0",
        "lsps1-max-initial-lsp-balance-sat": "1000000",
    }
