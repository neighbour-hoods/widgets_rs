import init, { run_app } from './pkg/paperz_ui.js';
async function main() {
   await init('/pkg/paperz_ui_bg.wasm');
   run_app();
}
main()
