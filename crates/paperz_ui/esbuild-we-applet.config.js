const esbuild = require('esbuild');
const { wasmLoader } = require('esbuild-plugin-wasm');

esbuild.build({
  entryPoints: ['./crates/paperz_ui/index.js'],
  bundle: true,
  outfile: './crates/paperz_ui/pkg/bundle-we-applet.js',
  format: 'esm',
  plugins: [
    wasmLoader({
        mode: 'embedded'
    })
  ]
}).then(_ => console.log("success 🚀"))
  .catch(() => process.exit(1))
