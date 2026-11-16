FROM debian:stable-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ngspice python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work
