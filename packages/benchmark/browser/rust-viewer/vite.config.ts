// Builds the Rust core's bench page into dist/, served at /rust/. The core
// runs in the doenetml-worker build, which the page loads from
// /doenetml-worker/ (served there by browser/serve.mjs and by
// packages/memory-benchmark).
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const prototype = resolve(__dirname, "../../../doenetml-prototype");
const { version } = JSON.parse(
    readFileSync(resolve(prototype, "package.json"), "utf8"),
);

export default defineConfig({
    root: __dirname,
    base: "/rust/",
    plugins: [react()],
    define: { DOENETML_VERSION: JSON.stringify(version) },
    build: { outDir: "dist", emptyOutDir: true, target: "esnext" },
});
