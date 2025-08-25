FROM alpine:3.19

ARG USER=root
ARG USER_UID=
ARG USER_GID=
ARG DOCKER_GID=

# install basic packages
RUN echo 'http://dl-cdn.alpinelinux.org/alpine/edge/community/' >> /etc/apk/repositories
RUN apk update
RUN apk add --no-cache \
  bash                 \
  bash-completion      \
  less                 \
	sudo                 \
  openssh              \
  git                  \
  tmux                 \
  ripgrep              \
  kakoune              \
  build-base

# set up dev user
RUN [ ! -z $USER_UID ] && addgroup -g $USER_GID $USER || :
RUN [ ! -z $USER_UID ] && adduser -u $USER_UID -G $USER -D -h /home/$USER $USER || :
RUN [ ! -z $DOCKER_GID ] && addgroup -g $DOCKER_GID docker || :
RUN [ ! -z $DOCKER_GID ] && addgroup $USER docker || :
RUN [ ! -z $USER ] && echo "$USER ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers || :
USER $USER
RUN echo 'export PS1="\w\n\[\e[;33m\]>\[\e[m\] "' >> $HOME/.profile
RUN echo 'export EDITOR="kak"' >> $HOME/.profile
RUN echo '. /usr/share/bash-completion/bash_completion' >> $HOME/.profile

# add claude
RUN sudo apk add --no-cache npm
RUN sudo npm install -g @anthropic-ai/claude-code

# set up rust for dev user
RUN sudo apk add --no-cache rustup
RUN /usr/bin/rustup-init -y
