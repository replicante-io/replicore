Replicante
==========
A datastore automation system.


Organization
============
This repo homes the following:

  * All the documentation in `docs/`
    * Special mention: agent and datastore specifications in `docs/specs`.


Playgrounds
===========
Playgrounds are docker and docker-compose projects that run distributed
datastores locally so that replicante can be developed and tested.

They moved to a dedicated repo: https://github.com/replicante-io/playgrounds


Development Documentation
=========================
The code is documented with `rustdoc` even for private methods.
This helps existing and new developers keep a handle on the codebase.

Use the following command from the root of the repo to generate the documentation:
```bash
cargo rustdoc -- --document-private-items
```

TODOs and the like
==================
You can scan the code for `TODO`s, `NOTE`s, etcetera with `fixme`.

```bash
npm install fixme
node_modules/.bin/fixme -i 'devtools/**' -i 'node_modules/**' -i 'target/**' '**/*.rs'
rm -r node_modules package-lock.json
```


Build dependences
=================

  * cmake
  * openssl-devel
