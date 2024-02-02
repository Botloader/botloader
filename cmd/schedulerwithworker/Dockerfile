# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin scheduler
RUN cargo build --release --bin vmworker

#run
FROM debian:bookworm AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/scheduler /usr/local/bin/botloader-scheduler
COPY --from=builder /app/target/release/vmworker /usr/local/bin/botloader-vmworker

RUN apt-get update
RUN apt-get install ca-certificates -y

ENV BOT_RPC_LISTEN_ADDR="0.0.0.0:7448"
EXPOSE 7448

ENV BL_VMWORKER_BIN_PATH="/usr/local/bin/botloader-vmworker"

# metrics
EXPOSE 7803 


ENTRYPOINT ["/usr/local/bin/botloader-scheduler"]