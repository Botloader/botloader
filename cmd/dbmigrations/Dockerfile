FROM rust:1.67

RUN cargo install sqlx-cli

WORKDIR /usr/src/myapp

COPY components/stores/migrations ./migrations

CMD ["sqlx", "migrate", "run"]
