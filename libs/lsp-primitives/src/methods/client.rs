use crate::json_rpc::{DefaultError, JsonRpcMethod, NoParams};
pub use crate::lsps0::common_schemas::{NetworkChecked, NetworkUnchecked};
pub use crate::lsps0::schema::ListprotocolsResponse;
pub use crate::lsps1::schema::{
    Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, Lsps1InfoRequest, Lsps1InfoResponse,
};
pub use crate::lsps2::schema::{
    Lsps2BuyError, Lsps2BuyRequest, Lsps2BuyResponse, Lsps2GetInfoError, Lsps2GetInfoRequest,
    Lsps2GetInfoResponse, Lsps2GetVersionsResponse,
};

pub type Lsps0ListProtocols = JsonRpcMethod<NoParams, ListprotocolsResponse, DefaultError>;
pub type Lsps1Info = JsonRpcMethod<Lsps1InfoRequest, Lsps1InfoResponse, DefaultError>;
pub type Lsps1CreateOrder = JsonRpcMethod<
    Lsps1CreateOrderRequest<NetworkChecked>,
    Lsps1CreateOrderResponse<NetworkUnchecked>,
    DefaultError,
>;
pub type Lsps2GetVersions = JsonRpcMethod<NoParams, Lsps2GetVersionsResponse, DefaultError>;
pub type Lsps2GetInfo = JsonRpcMethod<Lsps2GetInfoRequest, Lsps2GetInfoResponse, Lsps2GetInfoError>;
pub type Lsps2Buy = JsonRpcMethod<Lsps2BuyRequest, Lsps2BuyResponse, Lsps2BuyError>;

pub const LSPS0_LIST_PROTOCOLS: Lsps0ListProtocols =
    Lsps0ListProtocols::new("lsps0.list_protocols");

// LSPS1: Buy Channels
pub const LSPS1_GETINFO: Lsps1Info = Lsps1Info::new("lsps1.info");
pub const LSPS1_CREATE_ORDER: Lsps1CreateOrder = Lsps1CreateOrder::new("lsps1.create_order");

// LSPS2: JIT-channels
pub const LSPS2_GET_VERSIONS: Lsps2GetVersions = Lsps2GetVersions::new("lsps2.get_versions");
pub const LSPS2_GET_INFO: Lsps2GetInfo = Lsps2GetInfo::new("lsps2.get_info");
pub const LSPS2_BUY: Lsps2Buy = Lsps2Buy::new("lsps2.buy");
