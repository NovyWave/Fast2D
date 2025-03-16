FROM rust:1
WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo,from=rust,source=/usr/local/cargo \
    cargo install mzoon --git https://github.com/MoonZoon/MoonZoon --rev d3333db9ebe52c49117858927df5c73f9d9b2dae --locked

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo,from=rust,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    /usr/local/cargo/bin/mzoon build -r

RUN --mount=type=cache,target=target \
    ["cp", "./target/release/backend", "/usr/local/bin/moon_app"]

ENTRYPOINT ["moon_app"]
