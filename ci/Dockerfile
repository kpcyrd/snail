FROM rust:latest
ARG TARGET
WORKDIR /app
COPY ci/setup.sh ci/setup.sh
RUN ci/setup.sh
COPY . .
