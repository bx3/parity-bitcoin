# This Dockerfile uses Docker Multi-Stage Builds
# See https://docs.docker.com/engine/userguide/eng-image/multistage-build/

### Base Image
# Setup up a base image to use in Build and Runtime images
FROM rust:1.31 AS build

# rustup directory
ENV PATH=/root/.cargo/bin:$PATH \
    RUST_BACKTRACE=1

WORKDIR /build/parity-bitcoin
COPY . /build/parity-bitcoin

# install tools and dependencies
RUN apt-get update && \
        apt-get install -y --force-yes --no-install-recommends \
        g++ \
        build-essential \
        curl \
        git \
        file \
        binutils \
        ca-certificates \
        libssl-dev \
        pkg-config \
        libudev-dev \
        vim \
        python-pip \
        libpython2.7-stdlib

# show tools
RUN rustc -vV
RUN cargo -V
RUN gcc -v
RUN g++ -v
RUN pip install --upgrade pip
RUN pip install requests

# build pbtc
RUN cargo build -p pbtc
#RUN strip /build/parity-bitcoin/target/release/pbtc
#RUN file /build/parity-bitcoin/target/release/pbtc

# Runtime image, copies pbtc artifact from build image
#FROM ubuntu:16.04 AS run
#LABEL maintainer "Parity Technologies <devops@parity.io>"

#WORKDIR /pbtc-ubuntu
#COPY --from=build /build/parity-bitcoin/target/release/pbtc /pbtc-ubuntu/

EXPOSE 8333 18333 8332 18332 18443 18444
ENTRYPOINT ["/bin/bash"]
