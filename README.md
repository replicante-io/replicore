# Replicante
A distributed datastore orchestration system.

## Code of Conduct
Our aim is to build a thriving, healthy and diverse community.  
To help us get there we decided to adopt the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/)
for all our projects.

Any issue should be reported to [stefano-pogliani](https://github.com/stefano-pogliani)
by emailing [conduct@replicante.io](mailto:conduct@replicante.io).  
Unfortunately, as the community lucks members, we are unable to provide a second contact to report
incidents to.  
We would still encourage people to report issues, even anonymously.

In addition to the Code Of Conduct the following documents are relevant:

* The [Reporting Guideline](https://www.replicante.io/conduct/reporting), especially if you wish to
  report an incident.
* The [Enforcement Guideline](https://www.replicante.io/conduct/enforcing)

## Features and feature naming convention

Rust offers [features](https://doc.rust-lang.org/cargo/reference/features.html)
to conditionally code, logic and options into the final build.

In Replicante Core features are used to choose what to include with regards to things like:

* Backend implementations: such as stores, messaging platform, etc ... (WIP)
* Built-in orchestrator actions.
* Experimental or advanced features.

A naming convention for features is defined here to ensure consistency: `$namespace-$feature`.

* `$namespace`: logical grouping of features (for example `action` or `store_backend`).
* `$feature`: the exact feature (for example `debug` [actions] or `mongo` [store backend]).

The characters allowed in features name are restricted by rust and even more by <crates.io>:
<https://doc.rust-lang.org/cargo/reference/features.html#the-features-section>

As such the following two special characters should be the only one in use:

* `-` as the separator between `$namespace` and `$feature`.
* `_` as the world separator within each of `$namespace` and `$feature`.

## Development environment
Replicante Core requires a few dependencies in order to run.  
To make development easier and faster these dependencies are run locally using containers.

Replicante Core uses [podman](https://podman.io/) to manage these containers.  
This is mainly due to podman's support for cgroups v2 and rootless containers.

An opinionated development tool built on top of many generic projects is also available.
This is `replidev`, located in `devtools/replidev`, and is written in rust.
As mentioned, it will require some additional tools to work:

* [Podman](https://podman.io/)

It can be complied and installed in `$HOME/bin/replidev` with:

```bash
cargo install --path devtools/replidev
```

Once installed, the Replicante Core development environment can be set up with:

```bash
# Start the essential dependences (kafka, mongo, zookeeper)
# as well as an NGINX server for static content (inclues an "useful links" page).
$ replidev deps start essential
--> Create pod replideps-essential
3062d4295cbec38890f271b283f5d7aeae1e9987d865fc249685e48884608bf4
--> Start container replideps-essential-zookeeper
59cf21245a515905a52ccc545ae0b8efc16fb1a84f58a8e6bbe03685f9d1bffc
--> Start container replideps-essential-nginx
9c9be7b7368836fb7f2116a1d61d04dfbba0cc2ce9d0e4adb4ec5475162fb702
--> Start container replideps-essential-mongo
759980c6606aa46aefbf63f9f61dde7a471a10dded308ca1a79ef5cce6c2dbe2
--> Start container replideps-essential-kafka
e174144db830918a78805fbb729cfbd23b51568fedc5432d867abfa15f6186c1

# Once the command completes you can get the above mentioned links at
# http://localhost:8080/

#  When dependences start without any data, some initialisation may be required.
$ replidev deps initialise essential
--> Initialise essential/mongo from replideps-essential-mongo
==> Checking (and initialising) mongo replica set ...
MongoDB shell version v4.2.3
connecting to: mongodb://127.0.0.1:27017/?compressors=disabled&gssapiServiceName=mongodb
Implicit session: session { "id" : UUID("9d957838-7984-4d98-b667-e3b9937fb0d5") }
MongoDB server version: 4.2.3
---> Replica Set initialised, nothing to do
==> Ensuring all mongo indexes exist ...
MongoDB shell version v4.2.3
connecting to: mongodb://127.0.0.1:27017/?compressors=disabled&gssapiServiceName=mongodb
Implicit session: session { "id" : UUID("f72c6214-59c1-4066-baf6-78c19d61606d") }
MongoDB server version: 4.2.3

# Dependencies can be stopped, and containers removed with
$ replidev deps stop essential
--> Stop pod replideps-essential
3062d4295cbec38890f271b283f5d7aeae1e9987d865fc249685e48884608bf4
--> Remove pod replideps-essential
3062d4295cbec38890f271b283f5d7aeae1e9987d865fc249685e48884608bf4

# Once the pods have been stopped, persisted data can be PERMANENTLY deleted with
$ replidev deps clean essential --confirm
--> Clean data for essential pod (from ./devtools/data/essential)

# Additional development dependences and tools:
$ replidev deps list
NAME          STATUS    POD ID    DEFINITION   
essential     -         -         devtools/deps/podman/essential.yaml   
grafana       -         -         devtools/deps/podman/grafana.yaml   
jaeger        -         -         devtools/deps/podman/jaeger.yaml   
prometheus    -         -         devtools/deps/podman/prometheus.yaml   
sentry        -         -         devtools/deps/podman/sentry.yaml   
uis           -         -         devtools/deps/podman/uis.yaml
```

### Build dependences

* clang: for rdkafka
* cmake
* openssl-devel

## Playgrounds
Playgrounds are docker and docker-compose projects that run distributed
datastores locally so that replicante can be developed and tested.

They moved to a dedicated repo: <https://github.com/replicante-io/playgrounds>

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
