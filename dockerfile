FROM clux/muslrust:stable as build
RUN apt-get -yq update && apt-get -yqq install openssh-client

COPY . .

ENV HOMESERVER=https://example.com
ENV BOT_USERNAME=botto
ENV BOT_PASSWORD=muchsecretwow
ENV DB_URL=ws://example.com:8000
ENV DB_USER=botto
ENV DB_PASSWORD=muchsecret
ENV RUST_LOG=error,botto=trace

RUN eval `ssh-agent -s` && \
  cargo build --target x86_64-unknown-linux-musl --release

# copy important stuff to smaller base image
FROM alpine
COPY --from=build /volume/target/x86_64-unknown-linux-musl/release/botto /

CMD ["/botto"]
