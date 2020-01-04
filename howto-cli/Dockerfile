FROM alpine:latest

RUN apk --no-cache add ca-certificates

COPY target/release/howto-cli /usr/local/bin/howto

ENTRYPOINT [ "howto" ]
