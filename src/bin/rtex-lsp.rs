//! rtex Language Server — stdio JSON-RPC for `.tex` editor integration.

fn main() {
    if let Err(err) = rtex::run_lsp_server() {
        eprintln!("rtex-lsp error: {err}");
        std::process::exit(1);
    }
}
