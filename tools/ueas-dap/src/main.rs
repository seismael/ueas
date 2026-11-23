//! UEAS DAP binary entry point (`Content-Length` framed stdio).

fn main() -> anyhow::Result<()> {
    ueas_dap::run_dap_stdio()
}
