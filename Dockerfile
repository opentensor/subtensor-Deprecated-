FROM ubuntu:20.04

LABEL bittensor.image.authors="bittensor.com" \
	bittensor.image.vendor="Bittensor" \
	bittensor.image.title="bittensor/subtensor" \
	bittensor.image.description="Subtensor: The blockchain for the bittensor project." \
	bittensor.image.source="https://github.com/opentensor/subtensor.git" \
	bittensor.image.revision="${VCS_REF}" \
	bittensor.image.created="${BUILD_DATE}" \
	bittensor.image.documentation="https://opentensor.bittensor.io"


# show backtraces
ENV RUST_BACKTRACE 1

ARG DEBIAN_FRONTEND=noninteractive
# install tools and dependencies
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates cmake pkg-config libssl-dev git build-essential clang libclang-dev curl

# add substrate binary to docker image
RUN mkdir /subtensor
COPY . /subtensor

SHELL ["/bin/bash", "-c", "curl https://sh.rustup.rs -sSf | sh -s -- -y"]
SHELL ["/bin/bash", "-c", "source /root/.cargo/env"]
SHELL ["/bin/bash", "-c", "rustup toolchain install nightly-2020-10-06"]
SHELL ["/bin/bash", "-c", "rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06"]
SHELL ["/bin/bash", "-c", "WASM_BUILD_TOOLCHAIN=nightly-2020-10-06 cargo build --release"]
SHELL ["/bin/bash", "-c", "cd /subtensor"]
SHELL ["/bin/bash", "-c", "sudo ./install.sh"]

EXPOSE 30333 9933 9944
VOLUME ["/subtensor"]