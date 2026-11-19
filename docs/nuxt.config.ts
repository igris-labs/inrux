import { env } from "node:process";

const siteUrl = env.NUXT_SITE_URL || "https://inrux.igrislabs.com";

export default defineNuxtConfig({
  extends: ["@igrislabs/indocio"],
  modules: ["nuxt-gtag"],
  css: ["~/assets/main.css"],

  devtools: { enabled: true },

  ogImage: { enabled: false },
  aiReady: { enabled: false },
  sitemap: { enabled: true },

  nitro: { prerender: { routes: ["/"], crawlLinks: true, failOnError: true } },

  app: {
    head: {
      titleTemplate: "%s · Inrux",
      htmlAttrs: { lang: "en" },
      link: [{ rel: "icon", type: "image/svg+xml", href: "/favicon.svg" }],
      meta: [
        { name: "theme-color", content: "#FF671D" },
        {
          property: "og:image",
          content: new URL("/brand/social.png", siteUrl).href,
        },
      ],
    },
  },

  site: {
    url: siteUrl,
    name: "Inrux",
    description:
      "Learn to build native Rust interfaces with .iru components, reactive state and compiled styles.",
  },
});
