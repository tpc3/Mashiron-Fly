FROM rust:alpine as builder
WORKDIR /usr/src/mashiron-fly
COPY . .
RUN apk add --no-cache musl-dev && cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/mashiron-fly /bin/mashiron-fly
RUN mkdir /data
WORKDIR /data
CMD mashiron-fly
