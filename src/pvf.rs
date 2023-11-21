use anyhow::anyhow;
use futures::channel::oneshot;
use polkadot_node_core_pvf::{Config, PrepareJobKind, PvfPrepData, ValidationHost};
use polkadot_parachain_primitives::primitives::ValidationCode;
use polkadot_primitives::ExecutorParams;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

pub const NODE_VERSION: &'static str = env!("SUBSTRATE_CLI_IMPL_VERSION");

pub async fn setup_pvf_worker(pvfs_path: PathBuf) -> anyhow::Result<ValidationHost> {
    let prepare_worker_path = {
        // assuming they are both in ./target/release or at least in the same directory
        let mut path = std::env::current_exe().expect("current_exe failed?");
        path.pop();
        path.push("prechecker-worker");
        path
    };

    let prep_worker_version = Command::new(&prepare_worker_path)
        .args(["--version"])
        .output()
        .map_err(|err| {
            anyhow!(
                "Error executing prepare worker at '{:?}': {}",
                prepare_worker_path,
                err
            )
        })?
        .stdout;

    let prep_worker_version = std::str::from_utf8(&prep_worker_version)
        .expect("version is printed as a string; qed")
        .trim()
        .to_string();

    println!("Prechecker worker version: {}", prep_worker_version);

    let executor_worker_path = PathBuf::from("/dev/null");
    let (validation_host, worker) = polkadot_node_core_pvf::start(
        Config::new(
            pvfs_path,
            Some(NODE_VERSION.to_owned()),
            prepare_worker_path,
            executor_worker_path,
        ),
        Default::default(),
    )
    .await?;

    // CURSED
    let _detached_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(worker);
    });

    Ok(validation_host)
}

pub async fn precheck_pvf(
    mut validation_host: ValidationHost,
    pvf: ValidationCode,
    executor_params: ExecutorParams,
) -> anyhow::Result<Duration> {
    let raw_validation_code =
        sp_maybe_compressed_blob::decompress(&pvf.0, 12 * 1024 * 1024)?.to_vec();

    let pvf = PvfPrepData::from_code(
        raw_validation_code,
        executor_params,
        Duration::from_secs(60),
        PrepareJobKind::Prechecking,
    );

    let (tx, rx) = oneshot::channel();

    let now = Instant::now();

    validation_host
        .precheck_pvf(pvf.clone(), tx)
        .await
        .map_err(|e| anyhow!(e))?;

    rx.await?.map_err(|e| anyhow!("{:?}", e))?;

    Ok(now.elapsed())
}
