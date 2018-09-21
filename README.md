# Replicante
A datastore automation system.


## Developers Notebook
Architectural nodes, implementation details, suggestions, proposals and more.

The developers notebook is a collection of documents aimed at present and future project developers
as well as advanced users that want to know more about Replicante internals.
It is also a place to jot down ideas for the future or potential changes that may be
needed/usefull some day but are not quite yet.


## Development Documentation
The code is documented with `rustdoc` even for private methods.
This helps existing and new developers keep a handle on the codebase.

Use the following command from the root of the repo to generate the documentation:
```bash
cargo rustdoc -- --document-private-items
```


## Playgrounds
Playgrounds are docker and docker-compose projects that run distributed
datastores locally so that replicante can be developed and tested.

They moved to a dedicated repo: https://github.com/replicante-io/playgrounds


## Build dependences

  * cmake
  * openssl-devel


## TODOs and the like
You can scan the code for `TODO`s, `NOTE`s, etcetera with `fixme`.

```bash
npm install fixme
node_modules/.bin/fixme -i 'devtools/**' -i 'node_modules/**' -i 'target/**' '**/*.rs'
rm -r node_modules package-lock.json
```
