import { defineConfig } from "vite";

export default defineConfig({
  assetsInclude: ["**/*.wasm"],
  worker: {
    format: "es",
  },
  build: {
    target: "esnext",
  },
});
