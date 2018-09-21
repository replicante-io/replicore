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

const ArchitecturalNotes = {
  content: 'Architectural nodes, implementation details, suggestions, proposals and more',
  title: `[Architectural Notes](${siteConfig.baseUrl}docs/architecture)`,
};

const PrematureOptimisations = {
  content: 'Ideas and notes about performance improvements and possible features evolution',
  title: `[Premature Optimisations](${siteConfig.baseUrl}docs/optimizations)`,
};

const Dreamland = {
  content: 'With all the time in the world, what would you do?',
  title: `[Dreamland](${siteConfig.baseUrl}docs/dreamland)`,
};

class Index extends React.Component {
  render() {
    const language = this.props.language || '';

    return (
      <div>
        <div className="mainContainer">
          <Block layout="threeColumn">
            {[
              ArchitecturalNotes,
              PrematureOptimisations,
              Dreamland,
            ]}
          </Block>
        </div>
      </div>
    );
  }
}

module.exports = Index;
