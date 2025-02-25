FROM rust:1.85 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/smtp /usr/local/bin/app
ENV HOST=0.0.0.0
ENV PORT=${PORT:-2525}
CMD ["app"]