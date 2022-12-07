FROM rust:1.65 as builder
WORKDIR /usr/src/rusty-nussknacker
COPY src ./src
COPY js-sandbox ./js-sandbox
COPY Cargo.toml .
COPY Cargo.lock .
COPY build.rs ./build.rs
COPY benches ./benches
RUN mkdir snapshots
RUN cargo build 
#--profile release
#RUN cargo install --path .

FROM debian:bullseye-slim
ENV ROCKET_CONFIG=/Rocket.toml
COPY Rocket.toml /Rocket.toml
COPY --from=builder /usr/src/rusty-nussknacker/target/debug/rusty-nussknacker /usr/local/bin/rusty-nussknacker
ENTRYPOINT ["rusty-nussknacker"]