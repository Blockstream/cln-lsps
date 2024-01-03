import typing as t


def lsps1_server_options() -> t.Dict[str, t.Any]:
    return {
        "lsps1_enable": True,
        "lsps1_min_channel_balance_sat": "0",
        "lsps1_max_channel_balance_sat": "1000000",
        "lsps1_min_initial_client_balance_sat": "0",
        "lsps1_max_initial_client_balance_sat": "0",
        "lsps1_min_initial_lsp_balance_sat": "0",
        "lsps1_max_initial_lsp_balance_sat": "1000000",
    }
