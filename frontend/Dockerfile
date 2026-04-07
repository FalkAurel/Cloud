# =========================================
# Vue.js Development Container
# =========================================
ARG NODE_VERSION=24.12.0-alpine
FROM node:${NODE_VERSION}

# Set environment
ENV NODE_ENV=development
ENV NPM_CONFIG_LOGLEVEL=info

# Install common development dependencies
RUN apk add --no-cache \
    bash \
    git \
    curl \
    wget \
    vim \
    openssh \
    python3 \
    make \
    g++ \
    libc6-compat

# Install global Node tools for Vue development
RUN npm install -g npm@latest \
    && npm install -g @vue/cli

# Create workspace folder and set permissions
RUN mkdir -p /workspace && chown -R node:node /workspace
WORKDIR /workspace

# Use existing node user from the base image
USER node

# Expose the port typically used by Vite dev server
EXPOSE 5173

VOLUME [ "/data" ]

# Default command opens a shell, you can run 'npm create vue@latest' yourself
CMD [ "sh" ]