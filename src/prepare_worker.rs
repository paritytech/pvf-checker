pub use polkadot_node_core_pvf_common::decl_worker_main;

polkadot_node_core_pvf_common::decl_worker_main!(
    "prepare-worker",
    polkadot_node_core_pvf_prepare_worker::worker_entrypoint,
    env!("SUBSTRATE_CLI_IMPL_VERSION"),
    env!("SUBSTRATE_CLI_COMMIT_HASH"),
);
