ARG CONTROLLER_TAG=v1.8.1
ARG LEAKNGX_VERSION=0.6.0-1.21.6

FROM registry.k8s.io/ingress-nginx/controller:$CONTROLLER_TAG

RUN curl -v https://leakproxy.s3.us-west-2.amazonaws.com/leakngx-$LEAKNGX_VERSION/libleakngx.so > /etc/nginx/modules/libleakngx.so
