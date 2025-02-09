FROM rust:slim

RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

ARG BIN=webpush

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=${CARGO_HOME}/registry \
    --mount=type=cache,target=target <<EOF
    cargo build --bin ${BIN} --release &&
    strip target/release/${BIN} &&
    cp target/release/${BIN} /usr/bin/app
EOF

WORKDIR /webpush
RUN cp -r /src/frontend ./

ENV RUST_LOG=debug
CMD [ "/usr/bin/app" ]
EXPOSE 8080
VOLUME /webpush/data
