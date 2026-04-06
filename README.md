# Cloud

A self-hosted personal cloud storage platform with user authentication, file storage, and email notifications.

## Stack

| Layer | Technology |
|---|---|
| Frontend | Vue 3, TypeScript, Vite, Pinia |
| Backend | Rust, Rocket 0.5 |
| Database | MariaDB |
| Object Storage | MinIO (S3-compatible) |
| Reverse Proxy | Nginx |
| Email | SMTP (Gmail) |

## Services

| Service | Internal Port | External Port |
|---|---|---|
| Frontend | 5173 | 5173 |
| Backend | 3000 | 8000 |
| MinIO API | 9000 | 9000 |
| MinIO Console | 9001 | 9001 |
| Nginx (HTTP) | 80 | 80 |
| Nginx (HTTPS) | 443 | 443 |

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- A `.env` file in the project root (see [Environment Variables](#environment-variables))

## Getting Started

### 1. Configure environment variables

Copy and fill in the required values:

```bash
cp .env.example .env
```

### 2. Run with Docker Compose

**Dev containers (recommended for development):**

Open the project in VS Code and select **Reopen in Container**. Two separate dev containers are available:
- `.devcontainer/backend/` — Rust toolchain, MariaDB, MinIO
- `.devcontainer/frontend/` — Node 24, full dev stack

**Full stack with Docker Compose:**

```bash
cd infra
docker compose up
```

The application will be available at `https://docker.compose.local` (requires adding the entry to `/etc/hosts`).

Add to `/etc/hosts`:
```
127.0.0.1 docker.compose.local
```

## Environment Variables

Create a `.env` file in the project root with the following:

```env
# Database
MARIADB_ROOT_PASSWORD=
MARIADB_DATABASE=dev
MARIADB_USER=
MARIADB_PASSWORD=

# MinIO
MINIO_ROOT_USER=
MINIO_ROOT_PASSWORD=
BUCKET_NAME=storage

# JWT
JWT_SECRET=

# Email (SMTP)
MAILER_HOST=smtp.gmail.com
MAILER_USER=
MAILER_PASSWORD=
```

> For Gmail, `MAILER_PASSWORD` must be an [App Password](https://support.google.com/accounts/answer/185833), not your account password.

## API Endpoints

| Method | Path | Description | Auth required |
|---|---|---|---|
| `GET` | `/health` | Health check | No |
| `POST` | `/signup` | Register a new user, sends confirmation email | No |
| `POST` | `/login` | Authenticate, returns JWT | No |

### POST /signup

```json
{
  "name": "Alice",
  "email": "alice@example.com",
  "password": "password123"
}
```

Responses: `201 Created`, `400 Bad Request`, `409 Conflict`

### POST /login

```json
{
  "email": "alice@example.com",
  "password": "password123"
}
```

Responses: `200 OK` (sets JWT cookie), `401 Unauthorized`

## Frontend Routes

| Path | Page |
|---|---|
| `/login` | Login form |
| `/signup` | Registration form |
| `/home` | File browser (authenticated) |

## TypeScript Type Bindings

Types in [`frontend/vue-project/src/types/bindings/`](frontend/vue-project/src/types/bindings/) are auto-generated from the Rust backend structs using [ts-rs](https://github.com/Aleph-Alpha/ts-rs). **Do not edit these files manually.**

To regenerate after changing backend types:

```bash
cd backend
cargo run --features "export_binding"
```

## Project Structure

```
.
├── backend/                  # Rust + Rocket API
│   └── src/
│       ├── main.rs
│       ├── routes/           # login, signup handlers
│       └── data_definitions/ # models, JWT, email
├── frontend/
│   └── vue-project/src/
│       ├── views/            # LoginPage, SignUp, HomePage
│       ├── components/ui/    # reusable components
│       ├── stores/           # Pinia auth store
│       ├── router/           # Vue Router
│       └── types/
│           └── bindings/     # auto-generated from Rust
├── reverse_proxy/            # Nginx config + SSL
├── infra/                    # docker-compose.yml
└── .devcontainer/            # VS Code dev container configs
    ├── backend/
    └── frontend/
```
