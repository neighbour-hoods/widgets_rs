import { run_app } from './pkg/paperz_ui.js';
import { AppWebsocket, AdminWebsocket } from '@holochain/client';

async function main() {
  let element = document.getElementById("paperz_main");
  let admin_ws_js = await AdminWebsocket.connect("ws://localhost:9000");
  let app_ws_js = await AppWebsocket.connect("ws://localhost:9999");
  let app_info = await app_ws_js.appInfo({ installed_app_id: 'test-app' });
  let cell_id_js = app_info.cell_data[0].cell_id;
  run_app(element, admin_ws_js, app_ws_js, cell_id_js);
}
main()
