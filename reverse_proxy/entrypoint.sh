#!/bin/bash
set -e

CERT_DIR="/etc/nginx/certs"
CERT_FILE="$CERT_DIR/docker.compose.local.pem"
KEY_FILE="$CERT_DIR/docker.compose.local.key"

mkdir -p "$CERT_DIR"

openssl req -x509 -newkey rsa:4096 -keyout "$KEY_FILE" -out "$CERT_FILE" -sha256 -days 3650 -nodes \
  -subj "/C=DE/ST=Baden-Württemberg/L=Mannheim/O=MKU/CN=docker.compose.local" \
  -addext "subjectAltName=DNS:docker.compose.local,DNS:localhost,IP:127.0.0.1"

# Start nginx in foreground
nginx -g "daemon off;"
