import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite-plus";

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
        port: 8008,
        open: false,
        proxy: {
            "/api": {
                target: "http://localhost:8007",
                changeOrigin: true,
            },
            "/uploads": {
                target: "http://localhost:8007",
                changeOrigin: true,
            },
        },
    },
});
