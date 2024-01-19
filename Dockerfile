FROM alpine:3.19
RUN echo 'http://dl-cdn.alpinelinux.org/alpine/edge/community/' >> /etc/apk/repositories
RUN apk update
RUN apk add --no-cache \
  bash                 \
  openssh              \
  git                  \
  rustup               \
  build-base
RUN /usr/bin/rustup-init -y
RUN echo '. $HOME/.cargo/env' > $HOME/.bashrc
