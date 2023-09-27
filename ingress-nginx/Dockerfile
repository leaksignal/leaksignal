ARG CONTROLLER_TAG=v1.8.1
ARG LEAKNGX_VERSION=0.7.0-1.21.6-musl

FROM registry.k8s.io/ingress-nginx/controller:$CONTROLLER_TAG
ARG LEAKNGX_VERSION

RUN mkdir -p /usr/local/nginx/modules/
RUN curl -v https://leakproxy.s3.us-west-2.amazonaws.com/leakngx-$LEAKNGX_VERSION/libleakngx.so > /usr/local/nginx/modules/libleakngx.so
