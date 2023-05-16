use ::subxt::{OnlineClient, PolkadotConfig};
use anyhow::anyhow;
use polkadot_parachain::primitives::{Id as ParaId, ValidationCode};

#[subxt::subxt(runtime_metadata_path = "assets/kusama_metadata.scale")]
pub mod polkadot {}

pub async fn fetch_parachain_pvfs(
    rpc_url: String,
) -> anyhow::Result<Vec<(ParaId, ValidationCode)>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await?;
    let storage = api.storage().at_latest().await?;

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
