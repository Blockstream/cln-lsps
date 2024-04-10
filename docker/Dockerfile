# Use a base image with necessary dependencies for compiling
FROM ubuntu:latest as cln-downloader

RUN apt-get update && apt-get install -y curl
RUN curl -Lo lightning.tar.xz https://github.com/ElementsProject/lightning/releases/download/v24.02.2/clightning-v24.02.2-Ubuntu-20.04.tar.xz

FROM ubuntu:latest as bitcoin-downloader
RUN apt-get update && apt-get install -y curl
RUN curl -Lo bitcoin.tar.gz https://bitcoin.org/bin/bitcoin-core-25.0/bitcoin-25.0-x86_64-linux-gnu.tar.gz

FROM ubuntu:latest as rust-downloader
RUN apt-get update && apt-get install -y curl
RUN curl -Lo rustup.sh https://sh.rustup.rs

FROM rust:latest as plugin-compiler
ADD . /workspace
WORKDIR workspace

RUN cargo build --workspace

FROM ubuntu:latest as python-downloader

RUN apt-get update && apt-get install -y python3 python3-venv python-pip

ENV VENV_PATH=/opt/python/venv
RUN python3 -m venv $VENV_PATH
ENV PATH=$VENV_PATH/bin:$PATH
ADD requirements.txt requirements.txt
RUN pip install -r requirements.txt

FROM ubuntu:latest as runner
RUN apt-get update && apt-get install -y libsodium-dev sqlite3 libpq5 libsqlite3-dev xz-utils python3 curl build-essential python3-pip pkg-config libssl-dev

copy --from=rust-downloader rustup.sh /tmp/rustup.sh
ENV CARGO_HOME=/opt/cargo
ENV PATH=$CARGO_HOME/bin:$PATH
RUN sh /tmp/rustup.sh -y

RUN cargo install sqlx-cli

COPY --from=cln-downloader lightning.tar.xz /tmp/lightning.tar.xz
COPY --from=bitcoin-downloader bitcoin.tar.gz /tmp/bitcoin.tar.gz
COPY --from=python-downloader /opt/python/venv /opt/python/venv

ENV PATH=/opt/python/venv/bin:$PATH
RUN tar -xf /tmp/bitcoin.tar.gz && mv /bitcoin-25.0/bin/* /usr/bin/
RUN tar -xf /tmp/lightning.tar.xz

ADD . /workspace
WORKDIR /workspace

RUN make all











