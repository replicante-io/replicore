# Replicante
A datastore automation system.


## Development environment
At the root of this repo is a docker-compose file that will start
all dependences needed to develop Replicante core components.

The compose project uses docker volumes to persist data so that containers can be
stopped/recreated without loosing all development data.
The project also comes with tools to inspect the services to see what is in them:

  * [adminMongo](https://adminmongo.markmoffat.com/): manage MongoDB store at http://localhost:4321/


To start and initialise the services:
```bash
$ docker-compose up

# Initialise MongoDB store
$ docker-compose exec mongo mongo
> rs.initiate({_id: 'replistore', members: [{_id: 0, host: 'localhost:27017'}]})
> // Init script as https://github.com/replicante-io/playgrounds/blob/master/images/replicante/indexes.js
```


## Build dependences

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
