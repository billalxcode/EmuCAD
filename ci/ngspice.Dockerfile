# Dev-only image for scripts/ngspice_golden.sh --docker-image emucad-ngspice.
FROM debian:stable-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ngspice \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work
