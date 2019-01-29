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

const AgentProtocol = {
  content: 'A description of the Agents interface, a JSON encoded HTTP API',
  title: `[The Agent Protocol](${siteConfig.baseUrl}docs/agent)`,
};

const ChangeLog = {
  content: 'Notable changes to the Datastore Model and/or the Agent Protocol',
  title: `[Model Change Log](${siteConfig.baseUrl}docs/changelog)`,
};

const DatastoreModel = {
  content: 'A conceptual representation that detiles and defines all properties of a datastore',
  title: `[The Datastore Model](${siteConfig.baseUrl}docs/model)`,
};

class Introduction extends React.Component {
  render() {
    const href = `${siteConfig.baseUrl}docs/intro`;
    return (<div>
      <h2>
        <a href={href}>Introduction</a>
      </h2>
      <p>
        Datastores, even when very different if pourpose, often follow very similar
        approaches to clustering, high availability, failover, etcetera ...
      </p>
      <p>
        This pages detail the model of a datastore upon which Replicante is built.
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
              DatastoreModel,
              AgentProtocol,
              ChangeLog,
            ]}
          </Block>
        </div>
      </div>
    );
  }
}

module.exports = Index;
