FROM rust:1.59.0 as builder

# Install libudev for 'hidapi'
RUN apt-get update -y \
    && apt-get install -y libudev-dev

RUN rustup component add rustfmt

# Build cargo crates
RUN USER=root cargo new --bin place-order-ms
WORKDIR ./place-order-ms
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

# Build place-order-ms project
COPY . .
RUN rm ./target/release/deps/place-order-ms*
RUN cargo build --release

# Run-time container
FROM debian:buster-slim
ARG APP=/usr/src/app

# Set specific user for container security
ENV APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

RUN apt-get update -y \
    && apt-get install -y ca-certificates openssl

COPY --from=builder /place-order-ms/target/release/place-order-ms ${APP}/place-order-ms

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
CMD ["./place-order-ms"]