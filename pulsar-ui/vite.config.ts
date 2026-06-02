import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { templateCompilerOptions } from '@tresjs/core' // 1. Импортируем настройки TresJS

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
    plugins: [
        vue({
            // 2. Внедряем настройки компилятора, чтобы Vue понимал 3D-теги (<TresMesh> и т.д.)
            ...templateCompilerOptions
        })
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1421,
            }
            : undefined,
        watch: {
            // 3. tell Vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
}));