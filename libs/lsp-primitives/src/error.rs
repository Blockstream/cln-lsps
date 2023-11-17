pub fn map_json_rpc_error_code_to_str(code: i64) -> &'static str {
    match code {
        -32700 => "parsing_error",
        -32600 => "invalid_request",
        -32601 => "method_not_found",
        -32602 => "invalid_params",
        -32603 => "internal_error",
        -32099..=-32000 => "implementation_defined_server_error",
        _ => "unknown_error_code",
    }
}

#[cfg(test)]
mod test {
    use crate::error::map_json_rpc_error_code_to_str;

    #[test]
    fn test_map_json_rpc_error_code_to_str() {
        assert_eq!(map_json_rpc_error_code_to_str(12), "unknown_error_code");
        assert_eq!(map_json_rpc_error_code_to_str(-32603), "internal_error");
    }
}
