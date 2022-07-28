FROM rust:latest as builder
# 要求 Rust 在 scratch 镜像下运行
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /backend

COPY . ./
RUN ls -a
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest as ca-certificates
RUN apk add -U --no-cache ca-certificates

FROM scratch

WORKDIR /backend
COPY --from=ca-certificates /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/lib/libnss* /usr/lib/
COPY --from=builder /usr/lib/libresolv* /usr/lib/
COPY --from=builder /backend/target/x86_64-unknown-linux-musl/release/curriculum_board_backend .
COPY --from=builder /backend/static/cedict_ts.u8 ./static/

EXPOSE 11451

ENTRYPOINT ["/backend/curriculum_board_backend"]