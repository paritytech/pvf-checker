use std::path::PathBuf;
use std::time::{Duration, Instant};
use futures::channel::oneshot;
use futures::FutureExt;
use polkadot_node_core_pvf::{Config, PvfPrepData};

use polkadot_parachain::primitives::ValidationCode;

fn other_io_error(s: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, s)
}

pub async fn precheck_pvf(pvfs_path: PathBuf, pvf: ValidationCode) -> anyhow::Result<Duration> {
    // TODO: consider extracting worker setup out of the loop
    // FIXME: support non-default ExecutorParams
    let program_path = std::env::current_exe()?;
    let (mut validation_host, worker) =
        polkadot_node_core_pvf::start(Config::new(pvfs_path, program_path), Default::default());

    let raw_validation_code =
        sp_maybe_compressed_blob::decompress(&pvf.0, 12 * 1024 * 1024)?.to_vec();

    let pvf = PvfPrepData::from_code(
        raw_validation_code,
        Default::default(),
        Duration::from_secs(60),
    );

    let (tx, rx) = oneshot::channel();

    let task = async move {
        let now = Instant::now();

        validation_host
            .precheck_pvf(pvf.clone(), tx)
            .await
            .map_err(other_io_error)?;

        Result::<Duration, anyhow::Error>::Ok(now.elapsed())
    };

    rx.await?.unwrap();

    futures::pin_mut!(task);
    futures::pin_mut!(worker);

    futures::select! {
        result = task.fuse() => Ok(result?),
        _ = worker.fuse() => unreachable!(),
    }
}
