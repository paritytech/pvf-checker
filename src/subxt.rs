use ::subxt::{storage::Storage, utils::H256, OnlineClient, PolkadotConfig};
use anyhow::anyhow;
use parity_scale_codec::{Decode, Encode};
use polkadot_parachain::primitives::{Id as ParaId, ValidationCode};
use polkadot_primitives::ExecutorParams;

#[subxt::subxt(runtime_metadata_path = "assets/kusama_metadata.scale")]
pub mod polkadot {}

pub async fn fetch_parachain_pvfs(
    storage: &Storage<PolkadotConfig, OnlineClient<PolkadotConfig>>,
) -> anyhow::Result<Vec<(ParaId, ValidationCode)>> {
    let paraids_query = polkadot::storage().paras().parachains();
    let paraids = storage.fetch(&paraids_query).await?;
    let paraids = paraids.ok_or(anyhow!("parachains storage should be initialized"))?;

    let mut code_hashes = Vec::with_capacity(50);
    let mut pvfs = Vec::with_capacity(50);

    for para_id in paraids {
        let code_hash_query = polkadot::storage().paras().current_code_hash(&para_id);
        let code_hash = storage.fetch(&code_hash_query).await?;
        let code_hash = code_hash.ok_or(anyhow!("missing code hash for {:?}", para_id))?;
        let para_id = ParaId::from(para_id.0);

        code_hashes.push((para_id, code_hash));
    }

    for (para_id, code_hash) in code_hashes {
        let pvf_query = polkadot::storage().paras().code_by_hash(&code_hash);
        let pvf = storage.fetch(&pvf_query).await?;
        let pvf = pvf.ok_or(anyhow!("missing PVF for {:?}", para_id))?;
        let pvf = ValidationCode(pvf.0);

        pvfs.push((para_id, pvf));
    }

    Ok(pvfs)
}

pub async fn fetch_on_chain_data(
    rpc_url: String,
    at_block: Option<H256>,
) -> anyhow::Result<(Vec<(ParaId, ValidationCode)>, ExecutorParams)> {
    let api = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await?;
    let storage = if let Some(block_hash) = at_block {
        api.storage().at(block_hash)
    } else {
        api.storage().at_latest().await?
    };

    let pvfs = fetch_parachain_pvfs(&storage).await?;

    let session_index = storage
        .fetch(&polkadot::storage().paras_shared().current_session_index())
        .await?
        .ok_or(anyhow!("missing session index"))?;

    let executor_params = storage
        .fetch(
            &polkadot::storage()
                .para_session_info()
                .session_executor_params(session_index),
        )
        .await?
        .ok_or(anyhow!("missing executor params"))?;
    let encoded = executor_params.encode();
    // FIXME: we need to convert types between polkadot and subxt properly.
    let executor_params = ExecutorParams::decode(&mut encoded.as_slice())?;
    Ok((pvfs, executor_params))
}
