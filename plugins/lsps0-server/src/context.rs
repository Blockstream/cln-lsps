use cln_rpc::ClnRpc;
use cln_rpc::model::{IntoRequest, Request, Response};
use cln_rpc::primitives::RpcError;
use async_trait::async_trait;

#[async_trait]
pub trait ClnRpcTrait {

    pub async fn call(&mut self, req: Request) -> Result<Response, RpcError>;

    pub async fn call_typed<R: IntoRequest>(
        &mut self,
        request: R,
    ) -> Result<R::Response, RpcError> {
        Ok(self
            .call(request.into())
            .await?
            .try_into()
            .expect("CLN will reply correctly"))
    }
}

impl ClnRpcTrait for ClnRpc {

    async fn call(&mut self, req : Request) -> Result<Response, RpcError> {
        self.call(req).await
    }


}


struct Context<PluginState, Rpc : ClnRpcTrait> {
    state : PluginState,
    rpc : Rpc
}
