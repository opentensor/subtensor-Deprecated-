FROM ubuntu:20.04

LABEL io.parity.image.authors="bittensor.com" \
	io.parity.image.vendor="Bittensor" \
	io.parity.image.title="bittensor/subtensor" \
	io.parity.image.description="Subtensor: The blockchain for the bittensor project." \
	io.parity.image.source="https://github.com/opentensor/subtensor.git" \
	io.parity.image.revision="${VCS_REF}" \
	io.parity.image.created="${BUILD_DATE}" \
	io.parity.image.documentation="https://opentensor.bittensor.io"


# show backtraces
ENV RUST_BACKTRACE 1

ARG DEBIAN_FRONTEND=noninteractive
# install tools and dependencies
RUN apt update && apt install -y libssl1.1 ca-certificates cmake pkg-config libssl-dev git build-essential clang libclang-dev curl

# add substrate binary to docker image
RUN mkdir /subtensor
COPY . /subtensor

SHELL ["/bin/bash", "-c", "curl https://sh.rustup.rs -sSf | sh -s -- -y"]
SHELL ["/bin/bash", "-c", "source /root/.cargo/env"]
SHELL ["/bin/bash", "-c", "rustup toolchain install nightly-2020-10-06"]
SHELL ["/bin/bash", "-c", "rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06"]
SHELL ["/bin/bash", "-c", "WASM_BUILD_TOOLCHAIN=nightly-2020-10-06 cargo build --release"]
SHELL ["/bin/bash", "-c", "cd /subtensor && ./install.sh"]

# check if executable works in this containe
#RUN /usr/local/bin/node-subtensor --version

EXPOSE 30333 9933 9944
VOLUME ["/subtensor"]

#ENTRYPOINT ["/usr/local/bin/release/node-subtensor"]