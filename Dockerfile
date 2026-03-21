# Minimal runtime image.
# The web bundle and Linux server binary are prepared on the host before deployment
# and staged into dist/container/web.

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY dist/container/web/ /app/
RUN mkdir -p /app/data
COPY crates/app/data/openwok.db /app/data/openwok.db

ENV PORT=3000
ENV IP=0.0.0.0
ENV DATABASE_PATH=/app/data/openwok.db

EXPOSE 3000

ENTRYPOINT ["/app/openwok-app"]
