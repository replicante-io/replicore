const React = require('react');
const CompLibrary = require('../../core/CompLibrary.js');

const Container = CompLibrary.Container;
const GridBlock = CompLibrary.GridBlock;

const siteConfig = require(`${process.cwd()}/siteConfig.js`);

function docUrl(doc, language) {
  return `${siteConfig.baseUrl}docs/${language ? `${language}/` : ''}${doc}`;
}

const Block = props => (
  <Container
    padding={['bottom', 'top']}
    id={props.id}
    background={props.background}>
    <GridBlock align="center" contents={props.children} layout={props.layout} />
  </Container>
);


const ApiReference = {
  content: 'Access features and integrate with Replicante with an HTTP JSON API',
  title: `[API Reference](${siteConfig.baseUrl}docs/api)`,
};

const Features = {
  content: 'Feature showcase and details',
  title: `[Features](${siteConfig.baseUrl}docs/features)`,
};

const QuickStart = {
  content: 'Step-by-step guide to setup a local, docker based, playground to test out replicante core',
  title: `[Quick Start](${siteConfig.baseUrl}docs/quick-start)`,
};


class Introduction extends React.Component {
  render() {
    const href = `${siteConfig.baseUrl}docs/intro`;
    return (<div>
      <h2>
        <a href={href}>Introduction</a>
      </h2>
      <p>
        Welcome to safe datastore automation.
      </p>
      <p>
        Replicante is an open source data-driven automation system built specifically for datastores.
      </p>
    </div>);
  }
}

class Index extends React.Component {
  render() {
    const language = this.props.language || '';
    return (
      <div>
        <div className="mainContainer">
          <div className="container paddingTop">
            <div className="wrapper" style={{textAlign: 'center'}}>
                <Introduction />
            </div>
          </div>
          <Block key="links" layout="threeColumn">
            {[
              QuickStart,
              Features,
              ApiReference
            ]}
          </Block>
        </div>
      </div>
    );
  }
}

module.exports = Index;
