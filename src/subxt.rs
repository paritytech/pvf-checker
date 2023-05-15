use ::subxt::{storage::StorageKey, OnlineClient, PolkadotConfig};
use polkadot_parachain::primitives::ValidationCode;

#[subxt::subxt(runtime_metadata_path = "assets/kusama_metadata.scale")]
pub mod polkadot {}

pub async fn fetch_all_pvfs(rpc_url: String) -> anyhow::Result<Vec<(StorageKey, ValidationCode)>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await?;

    let code_hashes_query = polkadot::storage().paras().current_code_hash_root();

    let storage = api.storage().at_latest().await?;

    let mut iter = storage
        .iter(code_hashes_query, 50) // 50 at a time
        .await?;

    let mut code_hashes = Vec::with_capacity(50);
    let mut pvfs = Vec::with_capacity(50);

    while let Some((para_id, code_hash)) = iter.next().await? {
        code_hashes.push((para_id, code_hash));
    }

    for (para_id, code_hash) in code_hashes {
        let pvf_query = polkadot::storage().paras().code_by_hash(&code_hash);

        let pvf = storage.fetch(&pvf_query).await?;

        let pvf = pvf.expect(&format!(
            "missing PVF for para_id: 0x{}",
            hex::encode(&para_id)
        ));
        let pvf = ValidationCode(pvf.0);

        pvfs.push((para_id, pvf));
    }

    Ok(pvfs)
}
