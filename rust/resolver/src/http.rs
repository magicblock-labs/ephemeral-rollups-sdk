//! module for working with http requests to solana rpc endpoints

use std::sync::Arc;

use rpc::nonblocking::rpc_client::RpcClient;
use sdk::pubkey::Pubkey;

use crate::{
    account, DelegationStatus, DelegationsDB, ResolverResult, DELEGATION_PROGRAM_ID,
    DELEGATION_RECORD_SIZE,
};

pub async fn update_account_state(
    chain: Arc<RpcClient>,
    db: DelegationsDB,
    pubkey: Pubkey,
) -> ResolverResult<DelegationStatus> {
    let status = fetch_account_state(chain, pubkey).await?;
    let Some(mut entry) = db.get_async(&pubkey).await else {
        // shouldn't happen really, as we only invoke update_account_state after cache insertion
        tracing::warn!(%pubkey, "updating account state for untracked record");
        return Ok(DelegationStatus::Undelegated);
    };
    entry.get_mut().status = status;
    Ok(status)
}

pub async fn fetch_account_state(
    chain: Arc<RpcClient>,
    pubkey: Pubkey,
) -> ResolverResult<DelegationStatus> {
    let delegation_record = account::delegation_record_pda(&pubkey);
    let Ok(account) = chain.get_account(&delegation_record).await else {
        // RpcClient::get_account returns error for non existing accounts,
        // and non-existent delegation record is tantamount to undelegated state
        return Ok(DelegationStatus::Undelegated);
    };
    let is_delegated = account.owner == DELEGATION_PROGRAM_ID && account.lamports != 0;

    let status = if is_delegated {
        if account.data.len() != DELEGATION_RECORD_SIZE {
            tracing::warn!(size = account.data.len(), "wrong delegation record size");
            // NOTE: unclear what to do in such a situation, but practically speaking this can
            // happen only if ABI of delegation program has changed, and this version of library
            // hasn't accounted for that, which means we are in trouble anyway
            return Ok(DelegationStatus::Undelegated);
        }
        let mut buffer = [0; 32];
        // first 8 bytes is a discriminator, followed by 32 bytes
        // representing the validator identity
        buffer.copy_from_slice(&account.data[8..40]);
        let validator = Pubkey::new_from_array(buffer);
        DelegationStatus::Delegated(validator)
    } else {
        DelegationStatus::Undelegated
    };
    Ok(status)
}
