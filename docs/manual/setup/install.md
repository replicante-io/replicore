# Installation
This page documents the step to install replicante for production use.
It does NOT detail the installation process of the needed [dependences](deps.md).

If you are looking for a development/test/demo environment the
[quick start guide](../first-steps/quickstart.md) is where you can find all that.


## 1. Installing dependencies
The first step is to install all the required dependencies.
As the process depends on the chosen solution this guide does not cover how.

For production installation it is **strongly recommended** that all dependences
support and are installed and configured in highly available mode.

When using the recommended set of dependencies these guides may be of help:

  * Storage layer: MongoDB - https://docs.mongodb.com/v3.6/installation/
  * Distributed coordinator: Zookeeper - http://zookeeper.apache.org/doc/r3.4.12/zookeeperAdmin.html#sc_zkMulitServerSetup
  * Event streaming platform: Kafka - https://kafka.apache.org/11/documentation.html#quickstart


## 2. Installing from code
The following instructions where executed on a clean Fedora 28 install but should work for any Linux system:

```bash
# Install needed tools and dependencies.
dnf install cmake gcc git make openssl-devel

# Install rust and cargo with rustup.
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Get the code and compile replicante.
git clone --recursive https://github.com/replicante-io/replicante.git
cd replicante
cargo build --release

# Ensure the built binaries works.
target/release/replicante --version
target/release/replictl --version

# Binaries are ready for use!
cp target/release/replicante target/release/replictl replicante.example.yaml /path/to/install/location/
```


## 3. Store initialisation
Whatever your choice, the store needs some initialisation before it can be used.
The requirements depend on the selected store and are documented in the code:

  * MongoDB:
    * Requirements: https://github.com/replicante-io/replicante/blob/master/data/store/src/backend/mongo/mod.rs#L47
    * Playground example: https://github.com/replicante-io/playgrounds/blob/master/images/replicante/indexes.js

It is possible to verify the store configuration with [`replictl check store schema`](../replictl/checks.md).
`replictl` requires replicante to be [configured](./config.md) before the tests can work.


## 4. Next steps
Once the binaries are ready and the store is initialised it is time to [configure replicante](./config.md).
