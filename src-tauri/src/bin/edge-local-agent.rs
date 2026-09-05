#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    daarion_edge_client_lib::run_local_agent_pilot();
}
