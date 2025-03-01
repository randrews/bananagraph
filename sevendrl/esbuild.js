require('esbuild').build({
    entryPoints: ['src/app.js'],
    bundle: true,
    minify: false,
    outdir: 'build',
    format: 'esm',
    loader: {
        '.wasm': 'file'
    }
}).catch(() => process.exit(1))