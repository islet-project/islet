FROM rust:latest

RUN apt update

RUN apt install --yes --no-install-recommends \
    sudo

WORKDIR /root/source

COPY . .

RUN ./scripts/init.sh

RUN cd /root

RUN rm -rf source
