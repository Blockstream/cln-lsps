use anyhow::{anyhow, Context, Result};
use cln_rpc::model::requests::{TxdiscardRequest, TxprepareRequest, TxsendRequest};
use cln_rpc::primitives as rpc_primitives;
use cln_rpc::ClnRpc;
use lsp_primitives::lsps0::common_schemas::{FeeRate, PublicKey, SatAmount};
use std::str::FromStr;
use std::time::Duration;

use crate::cln::rpc_model::{
    FundChannelCancelRequest, FundChannelCompleteRequest,
    FundChannelCompleteResponse, FundChannelStartRequest, FundChannelStartResponse,
};
use crate::db::schema::Lsps1Channel;

pub struct ChannelDetails {
    pub(crate) peer_id: PublicKey,
    pub(crate) amount: SatAmount,
    pub(crate) feerate: Option<FeeRate>,
    pub(crate) close_to: Option<String>,
    pub(crate) push_msat: Option<SatAmount>,
    pub(crate) mindepth: Option<u8>,
    pub(crate) reserve: Option<SatAmount>,
}

#[derive(Debug, Default, Clone)]
struct ChannelOpenErrorData {
    peer_id: Option<PublicKey>,
    funding_address: Option<String>,
    txid: Option<String>,
}

#[derive(Debug)]
struct ChannelOpenError {
    data: ChannelOpenErrorData,
    error: Box<dyn std::error::Error + Send>,
}

impl std::fmt::Display for ChannelOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.error.fmt(f)
    }
}

impl std::error::Error for ChannelOpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}

impl ChannelOpenErrorData {
    fn wrap(&self, error: Box<dyn std::error::Error + Send>) -> ChannelOpenError {
        ChannelOpenError {
            data: self.clone(),
            error,
        }
    }
}

/// An fundchannel method that is guarantueed to either fail or succeed
/// within a predetermined amount of time.
///
/// The `fund_channel` rpc-method is not suited for implementing an
/// LSP-server because it cannot be cancelled.
///
/// When the LSP-server provides a refund it must be sure that the channel
/// cannot come trough.
///
/// A LSP-customer could order a channel and fail to respond
/// to any `channel_open` message. The LSP-server provides a refund
/// and must ensure it will not continue to open the channel after
/// providing the refund.
///
/// Note, that the `timeout` parameter is a lower-bound. The underlying implementation calls
/// `fundchannel_start`, `tx_prepare`, `fundchannel_complete` and `tx_send`. Each of those
/// methods is called with this time-out prameter
pub async fn fundchannel_fallible(
    rpc: &mut ClnRpc,
    channel_details: &ChannelDetails,
    timeout: Duration,
) -> Result<Lsps1Channel> {
    let rpc_id = rpc_primitives::PublicKey::from_str(&channel_details.peer_id.to_hex())
        .context("Invalid peer_id")?;

    let result =
        fundchannel_without_publishing_funding_transaction(rpc, channel_details, timeout).await;

    match result {
        Ok(channelopen_response) => {
            // Broadcast the funding transaction
            let txsend = TxsendRequest { txid:  channelopen_response.funding_tx.clone()};
            let _ = rpc.call_typed(&txsend).await?;
            Ok(channelopen_response)
        }
        Err(channel_open_error) => {
            log::warn!(
                "Failed to open channel to peer {:?}. fundchannel will be cancelled",
                channel_details.peer_id,
            );
            log::warn!("Error: {:?}", channel_open_error.error);

            if let Some(txid) = channel_open_error.data.txid {
                log::debug!("Discard funding transaction {:?}", txid);
                let txdiscard_request = TxdiscardRequest { txid };
                let _ = rpc.call_typed(&txdiscard_request).await;
            };

            if let Some(peer_id) = channel_open_error.data.peer_id {
                log::debug!("Cancelling channel open to peer {:?}", peer_id);
                let fundchannel_cancel_request = FundChannelCancelRequest { id: rpc_id };
                let _ = rpc.call_typed(&fundchannel_cancel_request).await;
            } else {
                log::debug!("Channel open was never initiated successfully");
            }

            Err(anyhow!("Failed to open channel to peer {:?}", rpc_id))
        }
    }
}

///
async fn fundchannel_without_publishing_funding_transaction(
    rpc: &mut ClnRpc,
    channel_details: &ChannelDetails,
    timeout: Duration,
) -> Result<Lsps1Channel, ChannelOpenError> {
    // Calculate when we should time-out
    let current_time = std::time::Instant::now();
    let timeout_time = current_time + timeout;

    // We need to capture details about our channel and reserved utxo's
    // to clean-up on failure
    let mut error_data = ChannelOpenErrorData::default();

    let rpc_id = rpc_primitives::PublicKey::from_str(&channel_details.peer_id.to_hex())
        .map_err(|_| error_data.wrap(anyhow!("peer_id is not a valid ECDSA public key").into()))?;
    let amount = rpc_primitives::Amount::from_sat(channel_details.amount.sat_value());

    let feerate = channel_details
            .feerate
            .clone()
            .map(|x| rpc_primitives::Feerate::PerKw(x.to_sats_per_kwu().try_into().unwrap()));

    // Do fundchannel_request
    let fundchannel_request: FundChannelStartRequest = FundChannelStartRequest {
        id: rpc_id,
        amount,
        feerate: feerate,
        close_to: channel_details.close_to.clone(),
        push_msat: channel_details
            .push_msat
            .map(|x| rpc_primitives::Amount::from_sat(x.sat_value())),
        mindepth: channel_details.mindepth,
        reserve: channel_details
            .reserve
            .map(|x| rpc_primitives::Amount::from_sat(x.sat_value())),
    };

    // Do fundchannel start and wrap it in a timeout
    let future = rpc.call_typed(&fundchannel_request);
    let timeout = timeout_time
        .checked_duration_since(std::time::Instant::now())
        .ok_or(error_data.wrap(anyhow!("Timeout in channel open").into()))?;

    log::debug!("Call fundchannel_start with a timeout of {} sec", timeout.as_secs());
    let fundchannel_response: FundChannelStartResponse = tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| error_data.wrap(anyhow!("Time-out in RPC-command: fundchannel_start").into()))?
        .map_err(|e| error_data.wrap(Box::new(e)))?;

    let funding_address= fundchannel_response.funding_address.clone();
    error_data.peer_id = Some(channel_details.peer_id.clone());
    error_data.funding_address = Some(funding_address.clone());

    // Create the funding transaction
    // It is an unsigned psbt
    let address = funding_address;
    log::debug!("Constructing the funding transaction");
    let txprepare_request: TxprepareRequest = TxprepareRequest {
        outputs: vec![rpc_primitives::OutputDesc {
            address: address,
            amount: amount,
        }],
        feerate: None,
        minconf: None,
        utxos: None,
    };
    let timeout = timeout_time
        .checked_duration_since(std::time::Instant::now())
        .ok_or(error_data.wrap(anyhow!("Timeout in channel open").into()))?;
    log::debug!("Call txprepare with a timeout of {} sec", timeout.as_secs());
    let txprepare_response: cln_rpc::model::responses::TxprepareResponse =
        tokio::time::timeout(timeout, rpc.call_typed(&txprepare_request))
            .await
            .map_err(|_| error_data.wrap(anyhow!("Time-out in RPC-command: txprepare").into()))?
            .map_err(|e| error_data.wrap(Box::new(e)))?;

    error_data.txid = Some(txprepare_response.txid.clone());

    // Get the commitment transaction from the peer
    log::debug!("Securing the commitment transaction from peer");
    let fundchannelcomplete_request = FundChannelCompleteRequest {
        id: rpc_id,
        psbt: txprepare_response.psbt,
    };
    let timeout = timeout_time
        .checked_duration_since(std::time::Instant::now())
        .ok_or(error_data.wrap(anyhow!("Timeout in channel open").into()))?;
    log::debug!("Call 'fundchannel_complete with a timeout of {} sec", timeout.as_secs());
    let fundchannelcomplete_response: FundChannelCompleteResponse =
        tokio::time::timeout(timeout, rpc.call_typed(&fundchannelcomplete_request))
            .await
            .map_err(|_| {
                error_data.wrap(anyhow!("Time-out in RPC-command: fundchannel_complete").into())
            })?
            .map_err(|e| error_data.wrap(Box::new(e)))?;

    if !fundchannelcomplete_response.commitments_secured {
        return Err(error_data.wrap(
            anyhow!(
                "Unexpected error: fundchannel_complete failed to secure commitment transaction"
            )
            .into(),
        ));
    }


    let outnum = txprepare_request.outputs.iter().position(|x| x.address == fundchannel_response.funding_address)
        .ok_or_else(|| error_data.wrap(anyhow!("Failed to find matching output in funding transaction").into()))?
        .try_into()
        .map_err(|e| error_data.wrap(anyhow!("Error in type-conversion of outnum: {}", e).into()))?;


    Ok(Lsps1Channel {
        funding_tx : txprepare_response.txid,
        outnum : outnum
    })
}
