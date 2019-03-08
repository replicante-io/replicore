# Replicante
A datastore automation system.


## Development environment
At the root of this repo is a docker-compose file that will start
all dependences needed to develop Replicante core components.

The compose project uses docker volumes to persist data so that containers can be
stopped/recreated without loosing all development data.
These services are:

  * [Kafka](https://kafka.apache.org/) 1.0+ for events streaming and task queues.
  * [MongoDB](https://www.mongodb.com/) 3.4+ for the storage layer.
  * [Zookeeper](https://zookeeper.apache.org/) 3.4+ for cluster coordination (and for use by kafka).

The project also comes with tools to inspect the services to see what is in them:

  * [adminMongo](https://adminmongo.markmoffat.com/): manage MongoDB store at http://localhost:4321/
  * [kafka-topics-ui](https://github.com/Landoop/kafka-topics-ui): inspect the content of kafka topics at http://localhost:8001/
  * [Prometheus](https://prometheus.io/): monitoring server at http://localhost:9090/
  * [ZooNavigator](https://github.com/elkozmon/zoonavigator): manage Zookeeper at http://localhost:8000/

Extra services that may not be useful for day to day work but may be helpful on occasions
are prodived by the `docker-compose-exta.yml` docker-compose override file.
These services are:

  * [Grafana](https://grafana.com/): dashboards and visualisation at http://localhost:3000/


To start and initialise the services:
```bash
$ docker-compose up
# Use this INSTEAD to also run the extra services:
# docker-compose -f docker-compose.yml -f docker-compose-extra.yml up

# Initialise MongoDB store
$ docker-compose exec mongo mongo
> rs.initiate({_id: 'replistore', members: [{_id: 0, host: 'localhost:27017'}]})
> // Init script as https://github.com/replicante-io/playgrounds/blob/master/images/replicante/indexes.js
```

### FAQ

  * **adminMongo wants me to create a connection**
    Ensure the mongo server is up and running and the replicante DB exists, then restart the UI:
    `docker-compose restart mongoui`.


### Build dependences

  * cmake
  * openssl-devel


## Playgrounds
Playgrounds are docker and docker-compose projects that run distributed
datastores locally so that replicante can be developed and tested.

They moved to a dedicated repo: https://github.com/replicante-io/playgrounds


## Development Documentation
The code is documented with `rustdoc` even for private methods.
This helps existing and new developers keep a handle on the codebase.

Use the following command from the root of the repo to generate the documentation:
```bash
cargo rustdoc -- --document-private-items
```


## Developers Notebook
Architectural nodes, implementation details, suggestions, proposals and more.

The developers notebook is a collection of documents aimed at present and future project developers
as well as advanced users that want to know more about Replicante internals.
It is also a place to jot down ideas for the future or potential changes that may be
needed/usefull some day but are not quite yet.


## TODOs and the like
You can scan the code for `TODO`s, `NOTE`s, etcetera with `fixme`.

```bash
npm install fixme
node_modules/.bin/fixme -i 'devtools/**' -i 'node_modules/**' -i 'target/**' '**/*.rs'
rm -r node_modules package-lock.json
```


## Code of Conduct
Our aim is to build a thriving, healthy and diverse community.  
To help us get there we decided to adopt the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/)
for all our projects.

Any issue should be reported to [stefano-pogliani](https://github.com/stefano-pogliani)
by emailing [conduct@replicante.io](mailto:conduct@replicante.io).  
Unfortunately, as the community lucks members, we are unable to provide a second contact to report incidents to.  
We would still encourage people to report issues, even anonymously.

In addition to the Code Of Conduct below the following documents are relevant:

  * The [Reporting Guideline](https://www.replicante.io/conduct/reporting), especially if you wish to report an incident.
  * The [Enforcement Guideline](https://www.replicante.io/conduct/enforcing)
