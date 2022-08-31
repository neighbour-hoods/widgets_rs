import init, { run_app } from './pkg/trailz_ui.js';
import { AppWebsocket, AdminWebsocket } from '@holochain/client';

async function main() {
  await init('/pkg/trailz_ui_bg.wasm');
  let element = document.getElementById("trailz_main");
  let admin_ws_js = await AdminWebsocket.connect("ws://localhost:9000");
  let app_ws_js = await AppWebsocket.connect("ws://localhost:9999");
  let app_info = await app_ws_js.appInfo({ installed_app_id: 'trailz_main_zome' });
  let cell_id_js = app_info.cell_data[0].cell_id;
  run_app(element, admin_ws_js, app_ws_js, cell_id_js);
}
main()
