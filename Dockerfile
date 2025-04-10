

FROM rust:1.85 AS build

RUN USER=root cargo new --bin tdx-prover
WORKDIR /tdx-prover

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

# build for release
RUN rm ./target/release/deps/tdx-prover*
RUN cargo build --release

FROM rust:1.85

COPY --from=build /tdx-prover/target/release/tdx-prover .

CMD ["./tdx-prover"]
