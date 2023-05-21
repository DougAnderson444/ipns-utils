import { defineConfig } from "vite"

/** @type {import('vite').UserConfig} */
export default defineConfig({
    resolve: {
        alias: {
            process: "process/browser",
            util: "util",
        },
    },
    optimizeDeps: {
        esbuildOptions: {
            define: {
                global: "globalThis",
            },
        },
    },
})
