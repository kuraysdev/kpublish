FROM rust:1.75

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/kpublish"]