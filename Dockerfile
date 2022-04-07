#FROM debian:buster-slim
#RUN apt-get install -y pkg-config libssl-dev
#
#FROM rust:1.57.0-alpine as builder
#RUN apk add --no-cache musl-dev
#WORKDIR /opt
#RUN USER=root cargo new --bin poms
#WORKDIR /opt/poms
#COPY ./Cargo.toml ./Cargo.toml
#
#RUN cargo build --release
#RUN rm ./src/*.rs
#RUN rm ./target/release/deps/poms*
#
#ADD ./src ./src
#RUN cargo build --release
#
#FROM scratch
#WORKDIR /opt/poms
#COPY --from=builder /opt/poms/target/release/poms .
#
#EXPOSE 5000
#CMD ["/opt/poms/poms"]

FROM rust:1.59.0 as builder

# Install libudev for 'hidapi'
RUN apt-get update -y \
    && apt-get install -y libudev-dev

RUN rustup component add rustfmt
# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/place-order-ms"]