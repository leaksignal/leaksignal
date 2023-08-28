ARG CONTROLLER_TAG=v1.8.1
ARG CONTROLLER_HASH=sha256:e5c4824e7375fcf2a393e1c03c293b69759af37a9ca6abdb91b13d78a93da8bd
ARG LEAKNGX_VERSION=0.4.0-1.21.6

FROM registry.k8s.io/ingress-nginx/controller:$CONTROLLER_TAG@$CONTROLLER_HASH

RUN curl -v https://leakproxy.s3.us-west-2.amazonaws.com/leakngx-$LEAKNGX_VERSION/libleakngx.so > /etc/nginx/modules/libleakngx.so
