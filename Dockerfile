FROM alpine:3.19
ARG USER
ARG UID
ARG GID

RUN addgroup -g $GID $USER
RUN adduser -u $UID -G $USER -D -h /home/$USER $USER
RUN apk update
RUN apk add --no-cache sudo
RUN echo "$USER ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

RUN echo 'http://dl-cdn.alpinelinux.org/alpine/edge/community/' >> /etc/apk/repositories
RUN apk add --no-cache \
  bash                 \
  less                 \
  openssh              \
  git                  \
  just                 \
  rustup               \
  build-base

USER $USER
RUN /usr/bin/rustup-init -y
RUN $HOME/.cargo/bin/cargo install bacon
RUN echo '. $HOME/.cargo/env' > $HOME/.bashrc
