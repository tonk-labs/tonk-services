FROM rust:1.68-buster as build

RUN USER=root cargo new --bin tonk-web-server
WORKDIR ./tonk-web-server

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

# 5. Build for release.
RUN rm ./target/release/deps/tonk_web_server*;
RUN cargo build --release

# deploy stage
FROM debian:buster-slim

# # create app directory
RUN mkdir app 
WORKDIR /app

# # copy binary and configuration files
COPY --from=build /tonk-web-server/target/release/tonk-web-server ./

EXPOSE 8081 

# # set the startup command to run your binary
CMD ["/app/tonk-web-server"]
