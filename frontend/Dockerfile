####################################
# Build book
####################################
FROM debian:bullseye as build-book

RUN apt-get update
RUN apt-get install -y curl

WORKDIR /app/book
COPY book/ .

# Download mdbook

RUN curl -L -o mdbook.tar.gz https://github.com/rust-lang/mdBook/releases/download/v0.4.15/mdbook-v0.4.15-x86_64-unknown-linux-gnu.tar.gz
RUN tar xvf mdbook.tar.gz

# Build book
RUN ./mdbook build

####################################
# Build docs
####################################
FROM node:16-alpine as build-docs

WORKDIR /app/components/runtime/docgen
ENV PATH /app/node_modules/.bin:$PATH

# grab deps
COPY components/runtime/docgen/package.json ./
COPY components/runtime/docgen/package-lock.json ./

RUN npm ci

# copy the  source
COPY components/runtime/ ../

RUN npm run build

####################################
# Build frontend
####################################
FROM node:16-alpine as build-main

# build .d.ts
WORKDIR /app/types

COPY components/runtime/src/ts ./

RUN npm install typescript
RUN node node_modules/typescript/bin/tsc --build tsconfig.json
RUN cp -r globals typings
RUN tar -cf typings.tar typings/

# build frontend-common
WORKDIR /app/frontend-common

COPY frontend-common/package.json ./
COPY frontend-common/package-lock.json ./

RUN npm ci

COPY frontend-common/ ./

# Build frontend
WORKDIR /app/frontend

COPY frontend/package.json ./
COPY frontend/package-lock.json ./

RUN npm ci

COPY frontend/ ./
COPY --from=build-docs /app/components/runtime/docgen/docs public/docs
COPY --from=build-book /app/book/book public/book
RUN cp -r /app/types/typings.tar public/typings.tar

# Build config
ARG BOTLOADER_API_BASE
ENV REACT_APP_BOTLOADER_API_BASE=$BOTLOADER_API_BASE

ARG BOTLOADER_CLIENT_ID
ENV REACT_APP_BOTLOADER_CLIENT_ID=$BOTLOADER_CLIENT_ID

RUN npm run build

####################################
# Entrypoint
####################################
FROM nginx:stable-alpine

COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build-main /app/frontend/build /usr/share/nginx/html

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]