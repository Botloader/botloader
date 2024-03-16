# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin backend

#run
FROM debian:bookworm AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend


RUN apt-get update
RUN apt-get install ca-certificates -y

RUN apt-get update
RUN apt-get install ca-certificates -y

ENV BL_BROKER_RPC_LISTEN_ADDR="0.0.0.0:7480"
EXPOSE 7480

ENV BL_BROKER_HTTP_API_LISTEN_ADDR="0.0.0.0:7449"
EXPOSE 7449

ENV BOT_RPC_LISTEN_ADDR="0.0.0.0:7448"
EXPOSE 7448

# scheduler metrics
EXPOSE 7803

# broker metrics
EXPOSE 7802

# webapi metrics
EXPOSE 7801

# webapi listen addr
EXPOSE 7447


ENTRYPOINT ["/usr/local/bin/backend"]