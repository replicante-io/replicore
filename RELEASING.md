# Releasing Replicante Core

All replicante projects must be released using the `replidev release` commands.
These commands will guide the you through release tasks,
automating the repetitive parts and performing checks along the way.

```bash
# Prepare the repository for release.
# This command will guide you to update changelogs and versions.
$ replidev release prep

# Commit any changes done during the prep phase.
$ git commit .

# Run checks to ensure the release is ready.
$ replidev release check

# Make sure you are logged into docker hub so the push can succeed:
$ podman login registry-1.docker.io

# Once all changes are committed and the checks pass publish the release.
# This will also publish any crate/docker image in the project and tag the current commit.
$ replidev release publish

# Push the release commit (if needed) and the release tag.
$ git push
$ git push --tags

# Create the new release in GitHub and attach the collected binaries to it.
```
