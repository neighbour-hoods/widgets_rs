import { run_app } from './pkg/paperz_ui.js';

const paperzApplet = {
  async appletRenderers(
    appWebsocket,
    adminWebsocket,
    weServices,
    appletAppInfo
  ) {
    const paperz_cell_id = appletAppInfo.cell_data.find(
      c => c.role_id === 'paperz'
    );
    return {
      full(element, registry) {
        element.innerHTML = `<div id="paperz_main"></div>`;
        run_app(element, adminWebsocket, appWebsocket, paperz_cell_id);
      },
      blocks: [],
    };
  },
};

export default paperzApplet;
