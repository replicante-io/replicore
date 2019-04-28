#########################
# Build replicante core #
#########################
ARG RUST_VERSION=1.34.0
FROM rust:$RUST_VERSION as builder

# Add the code and compile core.
COPY . /code
RUN cd /code && cargo build --release --locked


#####################################
# Package core into a smaller image #
#####################################
FROM debian:stretch-slim

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

# Install tini supervisor
ARG TINI_VERSION=v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini
ENTRYPOINT ["/tini", "--"]

# Copy binaries from builder to smaller image.
COPY --from=builder /code/target/release/replicante /opt/replicante/bin/replicante
COPY --from=builder /code/target/release/replictl /opt/replicante/bin/replictl

# Set up runtime environment as needed.
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME
CMD ["/opt/replicante/bin/replicante"]

# Validate binaries.
RUN /opt/replicante/bin/replicante --version \
    && /opt/replicante/bin/replictl --version
