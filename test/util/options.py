import typing as t


def lsps1_server_options() -> t.Dict[str, t.Any]:
    return {
        "lsps1_min_capacity": "0",
        "lsps1_max_capacity": "100000000",
    }
