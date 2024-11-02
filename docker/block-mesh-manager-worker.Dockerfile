FROM ubuntu:22.04 AS base
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG DEBIAN_FRONTEND=noninteractive
RUN echo "BUILDPLATFORM = $BUILDPLATFORM"
RUN echo "TARGETPLATFORM = $TARGETPLATFORM"
RUN apt-get update
RUN apt-get install curl gzip git-all -y
FROM base AS build
WORKDIR /opt/
RUN curl -sLO https://github.com/block-mesh/block-mesh-monorepo/releases/latest/download/block-mesh-manager-worker-x86_64-unknown-linux-gnu.tar.gz \
  && tar -xvf block-mesh-manager-worker-x86_64-unknown-linux-gnu.tar.gz \
  && mv target/release/block-mesh-manager-worker block-mesh-manager-worker \
  && chmod +x block-mesh-manager-worker
CMD ["/opt/block-mesh-manager-worker"]