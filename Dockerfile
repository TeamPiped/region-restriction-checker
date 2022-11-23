FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app/

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
      --mount=type=cache,target=/app/target/ \
      cargo build --release && \
      cp target/release/region-restriction-checker .

FROM debian:stable-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/

COPY --from=builder /app/region-restriction-checker .

CMD ["/app/region-restriction-checker"]
