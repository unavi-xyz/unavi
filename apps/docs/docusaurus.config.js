// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "The Wired Docs",
  tagline: "The Wired - An open and decentralized web-based metaverse platform.",
  url: "https://docs.thewired.space",
  baseUrl: "/",
  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",
  favicon: "img/Logo.png",
  titleDelimiter: " / ",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  plugins: [
    [
      "docusaurus-plugin-typedoc",

      /** @type {import('typedoc').TypeDocOptions} */
      {
        id: "engine",
        entryPoints: ["../../packages/engine/src/index.ts"],
        tsconfig: "../../packages/engine/tsconfig.json",
        excludePrivate: true,
        readme: "none",
        out: "../docs-api/engine",
        sourceLinkTemplate: "https://github.com/wired-labs/wired/blob/{gitRevision}/{path}#L{line}",
        sidebar: {
          categoryLabel: "Engine",
        },
      },
    ],
    [
      "docusaurus-plugin-typedoc",

      /** @type {import('typedoc').TypeDocOptions} */
      {
        id: "gltf-extensions",
        entryPoints: ["../../packages/gltf-extensions/src/index.ts"],
        tsconfig: "../../packages/gltf-extensions/tsconfig.json",
        excludePrivate: true,
        readme: "none",
        out: "../docs-api/gltf-extensions",
        sourceLinkTemplate: "https://github.com/wired-labs/wired/blob/{gitRevision}/{path}#L{line}",
        sidebar: {
          categoryLabel: "glTF Extensions",
        },
      },
    ],
    [
      "docusaurus-plugin-typedoc",

      /** @type {import('typedoc').TypeDocOptions} */
      {
        id: "protocol",
        entryPoints: ["../../packages/protocol/src/index.ts"],
        tsconfig: "../../packages/protocol/tsconfig.json",
        excludePrivate: true,
        readme: "none",
        out: "../docs-api/protocol",
        sourceLinkTemplate: "https://github.com/wired-labs/wired/blob/{gitRevision}/{path}#L{line}",
        sidebar: {
          categoryLabel: "Protocol",
        },
      },
    ],
    [
      "@docusaurus/plugin-content-docs",
      {
        id: "docs-api",
        path: "docs-api",
        routeBasePath: "api",
        sidebarPath: require.resolve("./sidebars.js"),
      },
    ],
  ],

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        blog: false,

        docs: {
          sidebarPath: require.resolve("./sidebars.js"),
          editUrl: "https://github.com/wired-labs/docs/tree/main",
          routeBasePath: "/",
        },

        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      metadata: [
        { property: "og:image", content: "/img/Logo.png" },
        { name: "twitter:image", content: "/img/Logo.png" },
        { name: "twitter:card", content: "summary" },
      ],

      navbar: {
        title: "The Wired",
        logo: {
          alt: "Logo",
          src: "img/Logo-Dark.png",
        },
        items: [
          {
            to: "/",
            label: "Docs",
            position: "left",
            activeBaseRegex: `/docs/`,
          },
          {
            to: "/api",
            label: "API",
            position: "left",
            activeBaseRegex: `/docs-api/`,
          },
          {
            label: "Discord",
            href: "https://discord.gg/VCsAEneUMn",
            position: "right",
            "aria-label": "Discord invite",
          },
          {
            href: "https://github.com/wired-labs/wired",
            position: "right",
            className: "header-github-link",
            "aria-label": "GitHub repository",
          },
        ],
      },

      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
