FROM ubuntu:22.04

COPY . /islet
COPY .git/ /islet/.git/

WORKDIR /islet

RUN ./scripts/deps/pkgs.sh
RUN ./scripts/deps/rust.sh
RUN ./scripts/deps/simulator.sh
