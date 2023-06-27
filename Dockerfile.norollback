FROM rust:1.70.0-buster as builder

ADD . /ord

RUN cd /ord && cargo build --release

# runtime
FROM debian:buster-slim

COPY --from=builder /ord/target/release/ord /usr/local/bin/

RUN apt-get update && apt-get install ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["ord"]
CMD ["server"]
