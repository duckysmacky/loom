FROM rust:1-slim-trixie AS types
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/.sqlx ./.sqlx

ENV SQLX_OFFLINE=true
ENV TS_RS_EXPORT_DIR=/build/types
RUN cargo test --lib --quiet export_bindings

FROM node:22-alpine AS build
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
COPY --from=types /build/types ./src/lib/types
RUN npx vite build

FROM nginx:1-alpine AS runtime
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/build /usr/share/nginx/html
EXPOSE 80
