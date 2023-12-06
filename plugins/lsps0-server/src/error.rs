enum CustomMsgError {
    InvalidParams(serde_json::Value),
    InternalServer,
    UnknownMethod(String)
}
