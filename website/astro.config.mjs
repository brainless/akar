import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import mdx from "@astrojs/mdx";
import siteConfig from "./src/site.config.ts";

export default defineConfig({
  site: "https://akar.dev",
  integrations: [mdx(), ...(siteConfig.features.sitemap ? [sitemap()] : [])],
  output: "static",
  trailingSlash: "always",
  build: {
    format: "directory",
    inlineStylesheets: "auto",
  },
  vite: {
    build: {
      cssMinify: true,
    },
  },
  image: {
    service: {
      entrypoint: "astro/assets/services/sharp",
    },
    format: ["avif", "webp"],
  },
  server: {
    port: 3000,
    host: true,
  },
  devToolbar: {
    enabled: true,
  },
});
