FROM rust:latest

RUN apt update

RUN apt install --yes --no-install-recommends \
    sudo

WORKDIR /root/source

COPY . .

RUN ./scripts/init.sh

RUN mv assets ..

WORKDIR /root

RUN rm -rf source
