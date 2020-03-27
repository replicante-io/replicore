# Replicante Development helper
A tool to speed up development of Replicante Core.

  * Podman play kube based dependencies.
    * With start/stop/rm/recreate commands.
    * Add a static nginx page with links to all other tools.
    * One of the pods is an initialiser process for all deps.
  * Podman play kube based WebUIs for the dependencies.
  * Command to delete and fix selinux context data path.
  * Command to build docker images locally.
  * Command to start a mongo RS with agents for tests?


## Envisioned use
```bash
# One-off "install" step
cd devtools/replidev
cargo build
mkdir -p target/bin
cp target/debug/replidev target/bin

# Check out https://direnv.net/ to see if this can be useful?
export PATH=$PATH:$PWD/target/bin

# Once in path (or by passing the path every time)
# Look into https://docs.rs/structopt/0.3.9/structopt/ for CLI parser
replidev deps start essential  # mongo, kafka, zookeeper, ...
replidev deps start uis sentry ... # start additional dependencies
replidev deps stop ... # podman stop + podman rm
replidev deps clean ... # delete data paths
replidev deps restart ... # stop + start
replidev images check vX.Y.Z # build container images (probably with buildah)
replidev images build vX.Y.Z # clean build (no cache) images
```

If/when needed add optional config file to store flexible options (paths to things).

Future expansion could look at cross-component support (core, images, webui, website).
Additional features around release automation would be a good expansion too.
