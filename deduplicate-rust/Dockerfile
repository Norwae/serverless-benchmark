FROM ubuntu:latest

RUN apt-get update ; \
    DEBIAN_FRONTEND=noninteractive apt-get install -y curl gcc musl-tools zip && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >/tmp/bootstraprust.sh && \
    chmod 755 /tmp/bootstraprust.sh && \
    mkdir -p /var/build/src && \
    echo 'fn main() { println!("Hello World") }' >/var/build/src/main.rs && \
    sh /tmp/bootstraprust.sh -y && \
    /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl

ENV PATH=${PATH}:/root/.cargo/bin/:/root/.rustup/bin

COPY Cargo.toml /var/build
# build all dependencies once
RUN cd /var/build && \
    cargo build --release --target x86_64-unknown-linux-musl
COPY src /var/build/src

RUN cd /var/build && \
    touch src/main.rs && \
    cargo build --release --target=x86_64-unknown-linux-musl

COPY bootstrap /tmp

RUN cp /var/build/target/x86_64-unknown-linux-musl/release/deduplicate-rust /tmp && \
    cd /tmp && \
    zip -r dist.zip . -i 'static/*' -i bootstrap -i deduplicate-rust