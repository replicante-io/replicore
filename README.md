# Replicante
A datastore orchestration system.


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


## Development environment
At the root of this repo is a docker-compose file that will start
all dependences needed to develop Replicante core components.

The compose project uses docker volumes to persist data so that containers can be
stopped/recreated without loosing all development data.
These services are:

  * [Kafka](https://kafka.apache.org/) 1.0+ for events streaming and task queues.
  * [MongoDB](https://www.mongodb.com/) 4.0 for the storage layer.
  * [Zookeeper](https://zookeeper.apache.org/) 3.4 for cluster coordination (and for use by kafka).

Additional services to support develoment and debugging are provided as additional
docker-compose configuration files located under `devtools/`.
Check them out to see what additional services are available.

To avoid typing the configuration files you need every times, docker-compose
provides a `.env` file where the `COMPOSE_FILE` can be set to a list of files.
This list indicates which services are "active".
In `.env.example` and in the snippet below the required dependences and monitoring
tools are active, everything else is optional.

To start and initialise the services:
```bash
# Create the .env file (the first time).
$ cp .env.example .env

# Start all the services (-d to start in the background).
$ docker-compose up #-d

# Initialise dependences.
$ devtools/initialise.sh
```

### FAQ

  * **adminMongo wants me to create a connection**
    Ensure the mongo server is up and running and the replicante DB exists, then restart the UI:
    `docker-compose restart mongoui`.


### Build dependences

  * clang: for rdkafka
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
