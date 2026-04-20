FROM rust:alpine AS builder

WORKDIR /app

RUN apk add --no-cache build-base cmake perl musl-dev pkgconfig openssl-dev openssl-libs-static

COPY ./src ./src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN adduser -D -H -s /sbin/nologin -u 10001 production_user
RUN cargo build --release && strip target/release/backend

FROM scratch

WORKDIR /app

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /app/target/release/backend backend

USER production_user

EXPOSE 3000

CMD [ "./backend" ]
