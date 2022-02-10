# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin discordbroker

#run
FROM debian:bullseye AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/discordbroker /usr/local/bin/botloader-discordbroker

RUN apt-get update
RUN apt-get install ca-certificates -y

ENV BL_BROKER_RPC_LISTEN_ADDR="0.0.0.0:7480"
EXPOSE 7480

ENV BL_BROKER_HTTP_API_LISTEN_ADDR="0.0.0.0:7449"
EXPOSE 7449

# metrics
EXPOSE 7802 


ENTRYPOINT ["/usr/local/bin/botloader-discordbroker"]