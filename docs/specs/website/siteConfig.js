// See https://docusaurus.io/docs/site-config for all the possible
// site configuration options.

const siteConfig = {
  title: 'Protocol and Model Specifications',
  tagline: 'Definition of the Datastore model and the agent protocol',
  url: 'https://www.replicante.io',
  baseUrl: '/docs/specs/',

  // Used for publishing and more
  projectName: 'replicante',
  organizationName: 'replicante-io',

  // For no header links in the top nav bar -> headerLinks: [],
  headerLinks: [{
    doc: 'intro',
    label: 'Docs'
  }, {
    href: 'https://www.replicante.io',
    label: 'Website'
  }],

  /* path to images for header/footer */
  //headerIcon: 'img/docusaurus.svg',
  //footerIcon: 'img/docusaurus.svg',
  //favicon: 'img/favicon.png',

  /* Colors for website */
  colors: {
    primaryColor: 'rgb(119, 101, 193)',
    secondaryColor: '#205C3B',
  },

  // This copyright info is used in /core/Footer.js and blog RSS/Atom feeds.
  copyright: `Copyright © ${new Date().getFullYear()} Stefano Pogliani`,

  highlight: {
    // Highlight.js theme to use for syntax highlighting in code blocks.
    theme: 'default',
  },

  // Add custom scripts here that would be placed in <script> tags.
  scripts: ['https://buttons.github.io/buttons.js'],

  // On page navigation for the current documentation page.
  onPageNav: 'separate',
  // No .html extensions for paths.
  cleanUrl: true,
};

module.exports = siteConfig;
