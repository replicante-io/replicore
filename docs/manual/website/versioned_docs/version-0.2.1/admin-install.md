---
id: version-0.2.1-admin-install
title: Installation
sidebar_label: Installation
original_id: admin-install
---

This page documents the step to install replicante for production use.
It does NOT detail the installation process of the needed [dependences](admin-deps.md).

If you are looking for a development/test/demo environment the
[quick start guide](quick-start.md) is where you can find all that.


## 1. Installing dependencies
The first step is to install all the required dependencies.
As the process depends on the chosen solution this guide does not cover how to do so.

For production installation it is **strongly recommended** that all dependences
are installed and configured in highly available mode.

When using the recommended set of dependencies these guides may be of help:

  * Storage layer: MongoDB - https://docs.mongodb.com/manual/installation/
  * Distributed coordinator: Zookeeper - http://zookeeper.apache.org/doc/current/zookeeperAdmin.html#sc_zkMulitServerSetup
  * Event streaming platform: Kafka - https://kafka.apache.org/documentation/#quickstart


## 2. Install

### From pre-built binaries
Pre-built binaries are helpful for users to get up and running quickly.
Unfortunately they require a good deal of effort from the community to be available for all
popular distributions that people my want to use.
The Replicante Community cannot afford to provide pre-built binaries for all popular
Linux distributions at this stage but we do want to provide something to make things
easier on people.

Pre-built binaries are available but they may not work for your system, in which case
we suggest you use an alternative install method.

```bash
# Grab the binaries for the version of your choice from GitHub:
VERSION=vX.Y.Z
wget https://github.com/replicante-io/replicante/releases/download/$VERSION/checksum.txt
wget https://github.com/replicante-io/replicante/releases/download/$VERSION/replicante-linux-64bits
wget https://github.com/replicante-io/replicante/releases/download/$VERSION/replictl-linux-64bits

# Verify the integrity of the binaries:
sha256sum --check checksum.txt

# Verify the binaries work:
mv replicante-linux-64bits replicante
mv replictl-linux-64bits replictl
chmod +x replicante replictl
./replicante --version
./replictl --version
```

### From code
The following instructions where executed on a clean Fedora 28 install but should work for any
Linux system given the correct package manager and package names:

```bash
# Install needed tools and dependencies.
dnf install -y cmake gcc gcc-c++ git make openssl-devel

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

### With Docker
A docker image with the pre-compiled core is also available:
https://hub.docker.com/r/replicanteio/replicante

To check the image works as expected:
```bash
docker pull replicanteio/replicante:v0
docker run --rm -it replicanteio/replicante:v0 replicante --version
docker run --rm -it replicanteio/replicante:v0 replictl --version
```


## 3. Store initialisation
Whatever your choice, the store needs some initialisation before it can be used.
The requirements depend on the selected store and are documented in the code:

  * MongoDB:
    * Requirements: https://github.com/replicante-io/replicante/blob/master/data/store/src/backend/mongo/mod.rs#L50
    * Playground example: https://github.com/replicante-io/playgrounds/blob/master/images/replicante/indexes.js

It is possible to verify the store configuration with [`replictl check store schema`](replictl-check.md).
`replictl` requires replicante to be [configured](admin-config.md) before the tests can work.


## 4. Next steps
Once the binaries are ready and the store is initialised it is time to [configure replicante](admin-config.md).
