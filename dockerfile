FROM clux/muslrust:stable as build
RUN apt-get -yq update && apt-get -yqq install openssh-client

COPY . .

ENV HOMESERVER=https://example.com
ENV BOT_USERNAME=botto
ENV BOT_PASSWORD=muchsecretwow
ENV DB_URL=ws://example.com:8000
ENV DB_USER=botto
ENV DB_PASSWORD=muchsecret
ENV RUST_LOG=warn,botto=info

RUN eval `ssh-agent -s` && \
  cargo build --target x86_64-unknown-linux-musl --release

# copy important stuff to smaller base image

FROM alpine

RUN mkdir /client_data
RUN mkdir -p /db_data/setup

COPY --from=build /volume/target/x86_64-unknown-linux-musl/release/botto /
COPY db_data/setup db_data/setup
COPY .surrealdb /.surrealdb
COPY avatar.jpg .

RUN ls -R db_data
RUN ls /

CMD ["/botto"]
