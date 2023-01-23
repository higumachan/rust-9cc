FROM ubuntu:latest

RUN apt update
RUN apt-get -y install build-essential curl
ENV RUST_VERSION stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH $PATH:/root/.cargo/bin
