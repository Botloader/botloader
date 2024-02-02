FROM rust:1.75

RUN cargo install sqlx-cli

WORKDIR /usr/src/myapp

COPY components/stores/migrations ./migrations

CMD ["sqlx", "migrate", "run"]
