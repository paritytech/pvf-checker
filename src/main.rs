use ::subxt::utils::H256;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod pvf;
mod subxt;

#[derive(Parser)]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Given the rpc url, fetch all of the parachain validation codes
    /// from the runtime and try to compile them using PVF workers.
    ///
    /// # Example
    ///
    /// ```bash
    /// cargo run --release -- pvf-check --rpc-url "wss://kusama-rpc.polkadot.io:443"
    /// ```
    PvfCheck {
        /// Url for an RPC node to query the relay chain runtime.
        ///
        /// Example:
        /// `wss://kusama-rpc.polkadot.io:443` or `http://localhost:9933/`
        #[clap(long)]
        rpc_url: String,

        /// A list of `ParaId`s to skip pre-checking for.
        #[clap(long)]
        skip: Vec<u32>,

        /// An optional block hash to query runtime info at.
        /// If `None`, the latest block will be used.
        #[clap(long)]
        at_block: Option<H256>,
    },
}

#[allow(missing_docs)]
#[derive(Debug, Parser)]
pub struct ValidationWorkerCommand {
    /// The path to the validation host's socket.
    #[arg(long)]
    pub socket_path: String,
    /// The path to the worker-specific temporary directory.
    #[arg(long)]
    pub worker_dir_path: String,
    /// Calling node implementation version
    #[arg(long)]
    pub node_impl_version: String,
}

async fn handle_pvf_check(
    rpc_url: String,
    skip: Vec<u32>,
    at_block: Option<H256>,
) -> anyhow::Result<()> {
    let artifacts = PathBuf::from(".artifacts");
    let _ = std::fs::create_dir_all(artifacts.as_path());

    let pvfs_path = artifacts.as_path().join("pvfs");
    let _ = std::fs::create_dir_all(&pvfs_path);

    print!("Fetching parachain PVFs...");
    let (pvfs, executor_params) = subxt::fetch_on_chain_data(rpc_url, at_block).await?;
    println!(" SUCCESS ({} PVFs)", pvfs.len());
    if executor_params == Default::default() {
        println!("Using default ExecutorParams");
    } else {
        println!("Using ExecutorParams: {executor_params:?}");
    }

    println!("Setting up PVF worker...");
    let validation_host = pvf::setup_pvf_worker(pvfs_path).await?;

    for (para_id, pvf) in pvfs {
        if skip.binary_search(&u32::from(para_id)).is_ok() {
            println!("Skipping {:?}", &para_id);
            continue;
        }
        print!("Pre-checking {:?}:", &para_id);
        let duration =
            pvf::precheck_pvf(validation_host.clone(), pvf, executor_params.clone()).await?;
        println!(" SUCCESS ({}ms)", duration.as_millis());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let cli = Cli::parse();

    match cli.commands {
        Commands::PvfCheck {
            rpc_url,
            mut skip,
            at_block,
        } => {
            skip.sort();
            rt.block_on(handle_pvf_check(rpc_url, skip, at_block))?;
        }
    }

    Ok(())
}
