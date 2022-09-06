// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "The Wired Docs",
  tagline: "The Wired - A web-based virtual worlds platform done right.",
  url: "https://docs.thewired.space",
  baseUrl: "/",
  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",
  favicon: "img/Logo-Maskable.png",
  titleDelimiter: " / ",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        blog: false,

        docs: {
          sidebarPath: require.resolve("./sidebars.js"),
          editUrl: "https://github.com/wired-labs/wired/tree/main/apps/docs",
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
        { property: "og:image", content: "/img/Logo-Maskable.png" },
        { name: "twitter:image", content: "/img/Logo-Maskable.png" },
      ],

      announcementBar: {
        id: "discord",
        content:
          '🎉 The Wired has entered early alpha! Join our discord <a target="_blank" rel="noopener noreferrer" href="https://discord.gg/VCsAEneUMn">here</a>.',
        backgroundColor: "#52daff",
        textColor: "#000000",
        isCloseable: true,
      },

      navbar: {
        title: "The Wired",
        logo: {
          alt: "The Wired Logo",
          src: "img/Logo-Maskable.png",
        },
        items: [
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
