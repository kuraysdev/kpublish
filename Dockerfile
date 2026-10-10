FROM rust:1.98.1-bookworm AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo test --locked && cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /build/target/release/kpublish /app/kpublish
COPY templates /app/templates
USER 65532:65532
EXPOSE 8777
CMD ["/app/kpublish"]
