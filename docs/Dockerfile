FROM node:18-alpine AS base
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm ci

FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .

RUN npm run build

FROM nginx:1.23.2

COPY --from=builder /app/build /usr/share/nginx/html/docs
COPY nginx_site.conf /etc/nginx/conf.d/default.conf

EXPOSE 80