import init, { run_app } from './pkg/holochain_client_wrapper_ui.js';
async function main() {
   await init('/pkg/holochain_client_wrapper_ui_bg.wasm');
   run_app();
}
main()
