// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'LeakSignal',
  tagline: 'Protecting Sensitive Data',
  url: 'https://leaksignal.com',
  baseUrl: '/docs/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.png',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath: '/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],
  scripts: [
    {
      src: 'https://www.googletagmanager.com/gtag/js?id=G-R4HG2EVRBN',
      async: true,
    },
    'https://www.leaksignal.com/docs/js/gtag.js',
    'https://www.leaksignal.com/docs/js/bigpicture.js',
  ],
  
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        defaultMode: 'dark',
        disableSwitch: true,
        respectPrefersColorScheme: false,
      },
      navbar: {
        title: '',
        logo: {
          alt: 'LeakSignal',
          src: 'img/logo.png',
          href: 'https://www.leaksignal.com/',
        },
        items: [
          {
            type: 'doc',
            docId: 'index',
            position: 'left',
            label: 'Getting Started',
          },
          {
            type: 'doc',
            docId: 'Policy/Policy',
            position: 'left',
            label: 'Policy',
          },
          {
            type: 'html',
            value: '<a href="https://www.leaksignal.com/" class="navbar__item navbar__link">Overview</a>',
            position: 'right',
          },
          {
            type: 'html',
            value: '<a href="https://www.leaksignal.com/blog" class="navbar__item navbar__link">Blog</a>',
            position: 'right',
          },
          // {
          //   type: 'html',
          //   value: '<a href="https://www.leaksignal.com/doc" class="navbar__item navbar__link">Docs</a>',
          //   position: 'right',
          // },
          {
            type: 'html',
            value: '<a href="https://www.leaksignal.com/contact/form" class="navbar__item navbar__link">Contact Us</a>',
            position: 'right',
          },
          {
            type: 'html',
            value: '<a href="https://app.leaksignal.com/" class="navbar__item navbar__link">Sign In</a>',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Getting Started',
                to: '/docs/',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Twitter',
                href: 'https://twitter.com/leaksignal',
              },
              {
                label: 'Slack',
                href: 'https://tinyurl.com/leaksignalslack',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Blog',
                to: 'https://www.leaksignal.com/blog',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/leaksignal/leaksignal',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} LeakSignal`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
