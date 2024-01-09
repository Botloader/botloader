# This is the base Docker file we use to build the executables
# this way we can reuse the image layer caching across all the executables
# since they mostly use the same ones anwyays
FROM lukemathwalker/cargo-chef:latest-rust-1.75.0-bullseye AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 

RUN apt update && apt install -y protobuf-compiler

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Install rustup
RUN rustup component add rustfmt


# Build application
COPY . .
