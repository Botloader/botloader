# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin webapi

#run
FROM debian:bookworm AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/webapi /usr/local/bin/botloader-webapi

RUN apt-get update
RUN apt-get install ca-certificates -y

EXPOSE 7447
EXPOSE 7801

ENTRYPOINT ["/usr/local/bin/botloader-webapi"]