FROM rust:1.86 AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY ciron-common ./ciron-common
COPY cirond ./cirond
COPY cironctl ./cironctl
COPY proto ./proto

RUN cargo build --release --bin cirond

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    python3 \
    nginx \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/cirond /usr/local/bin/cirond

COPY examples/nginx-webapp/app.py /app/app.py
COPY examples/nginx-webapp/nginx.conf /etc/nginx/nginx.conf
COPY examples/nginx-webapp/ciron.toml /etc/ciron/ciron.toml

WORKDIR /app

RUN chmod +x /app/app.py

EXPOSE 80 50051

CMD ["cirond", "-c", "/etc/ciron/ciron.toml"]
