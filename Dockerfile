FROM rust:1.88 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
COPY templates ./templates
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/portfolio-tracker .
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000
CMD ["./portfolio-tracker"]