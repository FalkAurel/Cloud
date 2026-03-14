#!/bin/bash
set -e

CERT_DIR="/etc/nginx/certs"
CERT_FILE="$CERT_DIR/docker.compose.local.pem"
KEY_FILE="$CERT_DIR/docker.compose.local.key"

mkdir -p "$CERT_DIR"

# Ephemeral key certificate generation: https://stackoverflow.com/questions/10175812/how-can-i-generate-a-self-signed-ssl-certificate-using-openssl
openssl req -x509 -newkey rsa:4096 -keyout "$KEY_FILE" -out "$CERT_FILE" -sha256 -days 3650 -nodes -subj "/C=DE/ST=Baden-Württemberg/L=Mannheim/O=MKU"

# Start nginx in foreground
nginx -g "daemon off;"
  