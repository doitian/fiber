use ckb_jsonrpc_types::Script;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[cfg(feature = "watchtower")]
use jsonrpsee::types::error::{ErrorObjectOwned, CALL_EXECUTION_FAILED_CODE};

use crate::fiber::{
    channel::{RevocationData, SettlementData},
    types::{Hash256, Privkey, Pubkey},
};
#[cfg(feature = "watchtower")]
use crate::rpc::context::RpcContext;
#[cfg(feature = "watchtower")]
use crate::watchtower::WatchtowerStore;

/// RPC module for watchtower related operations
#[cfg(feature = "watchtower")]
#[rpc(server)]
trait WatchtowerRpc {
    /// Create a new watched channel
    #[method(name = "create_watch_channel")]
    async fn create_watch_channel(
        &self,
        ctx: RpcContext,
        params: CreateWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Remove a watched channel
    #[method(name = "remove_watch_channel")]
    async fn remove_watch_channel(
        &self,
        ctx: RpcContext,
        params: RemoveWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update revocation
    #[method(name = "update_revocation")]
    async fn update_revocation(
        &self,
        ctx: RpcContext,
        params: UpdateRevocationParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update pending remote settlement
    #[method(name = "update_pending_remote_settlement")]
    async fn update_pending_remote_settlement(
        &self,
        ctx: RpcContext,
        params: UpdatePendingRemoteSettlementParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update settlement
    #[method(name = "update_local_settlement")]
    async fn update_local_settlement(
        &self,
        ctx: RpcContext,
        params: UpdateLocalSettlementParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Create preimage
    #[method(name = "create_preimage")]
    async fn create_preimage(
        &self,
        ctx: RpcContext,
        params: CreatePreimageParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Remove preimage
    #[method(name = "remove_preimage")]
    async fn remove_preimage(
        &self,
        ctx: RpcContext,
        params: RemovePreimageParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Get TLC status - check if TLCs are settled on-chain and get discovered preimages.
    /// This is used by Fiber node to sync TLC settlement status from the watchtower.
    #[method(name = "get_tlc_status")]
    async fn get_tlc_status(
        &self,
        ctx: RpcContext,
        params: GetTlcStatusParams,
    ) -> Result<GetTlcStatusResult, ErrorObjectOwned>;
}

/// ignore rpc-doc-gen
/// RPC client
#[rpc(client)]
trait WatchtowerRpc {
    /// Create a new watched channel
    #[method(name = "create_watch_channel")]
    async fn create_watch_channel(
        &self,
        params: CreateWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Remove a watched channel
    #[method(name = "remove_watch_channel")]
    async fn remove_watch_channel(
        &self,
        params: RemoveWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update revocation
    #[method(name = "update_revocation")]
    async fn update_revocation(
        &self,
        params: UpdateRevocationParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update pending remote settlement
    #[method(name = "update_pending_remote_settlement")]
    async fn update_pending_remote_settlement(
        &self,
        params: UpdatePendingRemoteSettlementParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Update settlement
    #[method(name = "update_local_settlement")]
    async fn update_local_settlement(
        &self,
        params: UpdateLocalSettlementParams,
    ) -> Result<(), ErrorObjectOwned>;

    /// Create preimage
    #[method(name = "create_preimage")]
    async fn create_preimage(&self, params: CreatePreimageParams) -> Result<(), ErrorObjectOwned>;

    /// Remove preimage
    #[method(name = "remove_preimage")]
    async fn remove_preimage(&self, params: RemovePreimageParams) -> Result<(), ErrorObjectOwned>;

    /// Get TLC status - check if TLCs are settled on-chain and get discovered preimages.
    /// This is used by Fiber node to sync TLC settlement status from the watchtower.
    #[method(name = "get_tlc_status")]
    async fn get_tlc_status(
        &self,
        params: GetTlcStatusParams,
    ) -> Result<GetTlcStatusResult, ErrorObjectOwned>;
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateWatchChannelParams {
    /// Channel ID
    pub channel_id: Hash256,
    /// Funding UDT type script
    pub funding_udt_type_script: Option<Script>,
    /// The local party's private key used to settle the commitment transaction
    pub local_settlement_key: Privkey,
    /// The remote party's public key used to settle the commitment transaction
    pub remote_settlement_key: Pubkey,
    /// The local party's funding public key
    pub local_funding_pubkey: Pubkey,
    /// The remote party's funding public key
    pub remote_funding_pubkey: Pubkey,
    /// Settlement data
    pub settlement_data: SettlementData,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveWatchChannelParams {
    /// Channel ID
    pub channel_id: Hash256,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRevocationParams {
    /// Channel ID
    pub channel_id: Hash256,
    /// Revocation data
    pub revocation_data: RevocationData,
    /// Settlement data
    pub settlement_data: SettlementData,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePendingRemoteSettlementParams {
    /// Channel ID
    pub channel_id: Hash256,
    /// Settlement data
    pub settlement_data: SettlementData,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateLocalSettlementParams {
    /// Channel ID
    pub channel_id: Hash256,
    /// Settlement data
    pub settlement_data: SettlementData,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatePreimageParams {
    /// Payment hash
    pub payment_hash: Hash256,
    /// Preimage
    pub preimage: Hash256,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemovePreimageParams {
    /// Payment hash
    pub payment_hash: Hash256,
}

/// Parameters for getting TLC settlement status from watchtower
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTlcStatusParams {
    /// List of TLCs to check, each containing channel_id and payment_hash
    pub tlcs: Vec<TlcQuery>,
}

/// A single TLC query containing channel ID and payment hash
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TlcQuery {
    /// Channel ID
    pub channel_id: Hash256,
    /// Payment hash
    pub payment_hash: Hash256,
}

/// Result of TLC status query from watchtower
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTlcStatusResult {
    /// List of TLCs that are confirmed settled on-chain
    pub settled_tlcs: Vec<TlcQuery>,
    /// List of (payment_hash, preimage) pairs for discovered preimages
    pub preimages: Vec<PreimageResult>,
}

/// A preimage result containing payment hash and preimage
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreimageResult {
    /// Payment hash
    pub payment_hash: Hash256,
    /// Preimage
    pub preimage: Hash256,
}

#[cfg(feature = "watchtower")]
pub struct WatchtowerRpcServerImpl<S> {
    store: S,
}

#[cfg(feature = "watchtower")]
impl<S> WatchtowerRpcServerImpl<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

#[cfg(feature = "watchtower")]
#[async_trait::async_trait]
impl<S> WatchtowerRpcServer for WatchtowerRpcServerImpl<S>
where
    S: WatchtowerStore + Send + Sync + 'static,
{
    async fn create_watch_channel(
        &self,
        ctx: RpcContext,
        params: CreateWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store.insert_watch_channel(
            ctx.node_id,
            params.channel_id,
            params.funding_udt_type_script.map(Into::into),
            params.local_settlement_key,
            params.remote_settlement_key,
            params.local_funding_pubkey,
            params.remote_funding_pubkey,
            params.settlement_data,
        );
        Ok(())
    }

    async fn remove_watch_channel(
        &self,
        ctx: RpcContext,
        params: RemoveWatchChannelParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store
            .remove_watch_channel(ctx.node_id, params.channel_id);
        Ok(())
    }

    async fn update_revocation(
        &self,
        ctx: RpcContext,
        params: UpdateRevocationParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store.update_revocation(
            ctx.node_id,
            params.channel_id,
            params.revocation_data,
            params.settlement_data,
        );
        Ok(())
    }

    async fn update_pending_remote_settlement(
        &self,
        ctx: RpcContext,
        params: UpdatePendingRemoteSettlementParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store.update_pending_remote_settlement(
            ctx.node_id,
            params.channel_id,
            params.settlement_data,
        );
        Ok(())
    }

    async fn update_local_settlement(
        &self,
        ctx: RpcContext,
        params: UpdateLocalSettlementParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store
            .update_local_settlement(ctx.node_id, params.channel_id, params.settlement_data);
        Ok(())
    }

    async fn create_preimage(
        &self,
        ctx: RpcContext,
        params: CreatePreimageParams,
    ) -> Result<(), ErrorObjectOwned> {
        use crate::fiber::hash_algorithm::HashAlgorithm;
        let CreatePreimageParams {
            payment_hash,
            preimage,
        } = params;

        if HashAlgorithm::supported_algorithms()
            .iter()
            .all(|algorithm| payment_hash != algorithm.hash(preimage).into())
        {
            return Err(ErrorObjectOwned::owned(
                CALL_EXECUTION_FAILED_CODE,
                "Wrong preimage",
                Option::<()>::None,
            ));
        }
        self.store
            .insert_watch_preimage(ctx.node_id, payment_hash, preimage);
        Ok(())
    }
    async fn remove_preimage(
        &self,
        ctx: RpcContext,
        params: RemovePreimageParams,
    ) -> Result<(), ErrorObjectOwned> {
        self.store
            .remove_watch_preimage(ctx.node_id, params.payment_hash);
        Ok(())
    }

    async fn get_tlc_status(
        &self,
        _ctx: RpcContext,
        params: GetTlcStatusParams,
    ) -> Result<GetTlcStatusResult, ErrorObjectOwned> {
        let mut settled_tlcs = Vec::new();
        let mut payment_hashes = Vec::new();

        for tlc in &params.tlcs {
            // Extract the first 20 bytes of the payment hash for settled check
            let payment_hash_prefix: [u8; 20] = tlc.payment_hash.as_ref()[0..20]
                .try_into()
                .expect("payment hash should be at least 20 bytes");

            // Check if TLC is settled on-chain
            if self
                .store
                .is_tlc_settled_with_prefix(&tlc.channel_id, &payment_hash_prefix)
            {
                settled_tlcs.push(TlcQuery {
                    channel_id: tlc.channel_id,
                    payment_hash: tlc.payment_hash,
                });
            }

            // Collect payment hashes for preimage lookup
            payment_hashes.push(tlc.payment_hash);
        }

        // Get preimages for the payment hashes
        let preimages = self
            .store
            .get_preimages(&payment_hashes)
            .into_iter()
            .map(|(payment_hash, preimage)| PreimageResult {
                payment_hash,
                preimage,
            })
            .collect();

        Ok(GetTlcStatusResult {
            settled_tlcs,
            preimages,
        })
    }
}
