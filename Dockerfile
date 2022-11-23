FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app/

COPY . .

RUN cargo build --release

FROM alpine:latest

WORKDIR /app/

COPY --from=builder /app/target/release/region-restriction-checker .

CMD ["./region-restriction-checker"]
