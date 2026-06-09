import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite-plus";

const BACKEND_PORT = 9800;
const WEB_DEV_PORT = 9801;

// https://vite.dev/config/
export default defineConfig({
    lint: { options: { typeAware: true, typeCheck: true } },
    fmt: { sortImports: {} },
    staged: {
        "*": "vp check --fix",
    },
    plugins: [
        tanstackRouter({ autoCodeSplitting: true }),
        viteReact({ jsxImportSource: "@emotion/react" }),
        tailwindcss(),
    ],
    resolve: {
        tsconfigPaths: true,
    },
    server: {
        host: "127.0.0.1",
        port: WEB_DEV_PORT,
        open: false,
        proxy: {
            "/api": {
                target: `http://127.0.0.1:${BACKEND_PORT}`,
                changeOrigin: true,
            },
            "/uploads": {
                target: `http://127.0.0.1:${BACKEND_PORT}`,
                changeOrigin: true,
            },
        },
    },
});
