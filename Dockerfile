#########################
# Build replicante core #
#########################
ARG RUST_VERSION=1.59.0
FROM rust:$RUST_VERSION as builder

# Add packages needed to build core.
# There is no need to cleanup as this is a builder image.
RUN apt-get update
RUN apt-get install -y clang

# Add the code and compile core.
COPY . /code
RUN cargo build --manifest-path /code/Cargo.toml --release --locked


#####################################
# Package core into a smaller image #
#####################################
FROM debian:bullseye-slim

# Create a replicante user to avoid using root.
ARG REPLI_GID=1616
ARG REPLI_GNAME=replicante
ARG REPLI_UID=1616
ARG REPLI_UNAME=replicante
RUN addgroup --gid $REPLI_GID $REPLI_GNAME \
    && adduser --disabled-login --disabled-password --system --uid $REPLI_UID --gid $REPLI_GID $REPLI_UNAME

# Install needed runtime dependencies.
RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y libssl1.1 \
    && apt-get clean all

# Copy binaries from builder to smaller image.
COPY --from=builder /code/target/release/replicante /opt/replicante/bin/replicante
COPY --from=builder /code/target/release/repliadm /opt/replicante/bin/repliadm
COPY --from=builder /code/target/release/replictl /opt/replicante/bin/replictl

# Set up runtime environment as needed.
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME
WORKDIR /home/replicante
CMD ["/opt/replicante/bin/replicante"]

# Validate binaries.
RUN /opt/replicante/bin/replicante --version \
    && /opt/replicante/bin/repliadm --version \
    && /opt/replicante/bin/replictl --version
