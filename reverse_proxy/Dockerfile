FROM nginx:stable-alpine-slim

# Copy your Nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Install openssl
RUN apk add --no-cache openssl bash

# Copy a startup script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
