FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app/

COPY . .

RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app/

COPY --from=builder /app/target/release/region-restriction-checker .

CMD ["./region-restriction-checker"]
