FROM rust:alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev perl make postgresql-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && touch src/lib.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/*.rs

COPY . .
RUN cargo build --release

FROM alpine
RUN addgroup -S program_user && adduser -S program_user -G program_user
COPY --from=builder /app/target/release/lemmy_know /usr/local/bin/
USER program_user

STOPSIGNAL SIGINT
CMD ["/usr/local/bin/lemmy_know"]
