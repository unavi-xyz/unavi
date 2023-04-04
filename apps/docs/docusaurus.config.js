// @ts-check

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
        out: "packages/engine",
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
        out: "packages/gltf-extensions",
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
        out: "packages/protocol",
        sourceLinkTemplate: "https://github.com/wired-labs/wired/blob/{gitRevision}/{path}#L{line}",
        sidebar: {
          categoryLabel: "Protocol",
        },
      },
    ],
    [
      "docusaurus-plugin-typedoc",

      /** @type {import('typedoc').TypeDocOptions} */
      {
        id: "react-client",
        entryPoints: ["../../packages/react-client/src/index.ts"],
        tsconfig: "../../packages/react-client/tsconfig.json",
        excludePrivate: true,
        out: "packages/react-client",
        sourceLinkTemplate: "https://github.com/wired-labs/wired/blob/{gitRevision}/{path}#L{line}",
        sidebar: {
          categoryLabel: "React Client",
        },
      },
    ],
    [
      "@docusaurus/plugin-content-docs",
      {
        id: "packages",
        path: "./docs/packages",
        routeBasePath: "/packages",
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
          editUrl: "https://github.com/wired-labs/wired/tree/main/apps/docs",
          path: "./docs/main",
          routeBasePath: "/",
          sidebarPath: require.resolve("./sidebars.js"),
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
            activeBaseRegex: `^(?!/packages.*$).*`,
          },
          {
            to: "/packages",
            label: "Packages",
            position: "left",
            activeBaseRegex: `/packages`,
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
