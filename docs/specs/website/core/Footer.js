const React = require('react');

class Footer extends React.Component {
  docUrl(doc, language) {
    const baseUrl = this.props.config.baseUrl;
    const lang = language && language !== 'en' ? language : '';
    return `${baseUrl}docs/${lang}${doc}`;
  }

  render() {
    return (
      <footer className="nav-footer" id="footer">
        <section className="sitemap" style={{textAlign: 'center'}}>
          <div>
            <h5>Docs</h5>
            <a href={this.docUrl('intro', this.props.language)}>
              Introduction
            </a>
            <a href={this.docUrl('model', this.props.language)}>
              The Datastore Model
            </a>
            <a href={this.docUrl('agent', this.props.language)}>
              The Agent Protocol
            </a>
            <a href={this.docUrl('changelog', this.props.language)}>
              Model Change Log
            </a>
          </div>
          <div>
            <h5>Community</h5>
            <a href="https://github.com/replicante-io">GitHub Organisation</a>
            <a href="https://www.replicante.io/">Official Website</a>
          </div>
        </section>

        <section className="copyright">{this.props.config.copyright}</section>
      </footer>
    );
  }
}

module.exports = Footer;
