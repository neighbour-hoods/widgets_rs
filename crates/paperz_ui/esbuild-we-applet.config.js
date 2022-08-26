const esbuild = require('esbuild');
const { wasmLoader } = require('esbuild-plugin-wasm');

esbuild.build({
  entryPoints: ['./crates/paperz_ui/index-pre.js'],
  bundle: true,
  outfile: './crates/paperz_ui/index.js',
  format: 'esm',
  plugins: [
    wasmLoader({
        mode: 'embedded'
    })
  ]
}).then(_ => console.log("success 🚀"))
  .catch(() => process.exit(1))
