# Cloud

[![CI](https://github.com/FalkAurel/Cloud/actions/workflows/ci.yml/badge.svg)](https://github.com/FalkAurel/Cloud/actions/workflows/ci.yml)

A self-hosted personal cloud storage platform with user authentication, file storage, and email notifications.

## Stack

| Layer | Technology |
| --- | --- |
| Frontend | Vue 3, TypeScript, Vite, Pinia |
| Backend | Rust, Rocket 0.5 |
| Database | MariaDB |
| Object Storage | MinIO (S3-compatible) |
| Reverse Proxy | Nginx |
| Email | SMTP (Gmail) |

## Services

| Service | Internal Port | External Port |
| --- | --- | --- |
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

```text
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
MARIADB_HOST=db

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

All application routes are versioned under `/v1`. The `/health` endpoint is unversioned.

| Method | Path | Description | Auth required |
| --- | --- | --- | --- |
| `GET` | `/health` | Health check | No |
| `POST` | `/v1/signup` | Register a new user, sends confirmation email | No |
| `POST` | `/v1/login` | Authenticate, sets JWT cookie | No |
| `POST` | `/v1/logout` | Clear JWT cookie | Yes |
| `GET` | `/v1/me` | Get current user profile | Yes |
| `DELETE` | `/v1/users/:id` | Delete a user account | Yes |

### POST /v1/signup

```json
{
  "name": "Alice",
  "email": "alice@example.com",
  "password": "password123"
}
```

Responses: `201 Created`, `400 Bad Request`, `409 Conflict`

### POST /v1/login

```json
{
  "email": "alice@example.com",
  "password": "password123"
}
```

Responses: `200 OK` (sets JWT cookie), `401 Unauthorized`

### POST /v1/logout

Requires a valid `jwt` cookie. Clears the cookie on success.

Responses: `200 OK`, `401 Unauthorized`

### GET /v1/me

Requires a valid `jwt` cookie. Returns the authenticated user's profile.

```json
{
  "name": "Alice",
  "email": "alice@example.com",
  "is_admin": false
}
```

Responses: `200 OK`, `401 Unauthorized`

### DELETE /v1/users/:id

Requires a valid `jwt` cookie. A user may delete their own account. Admins may delete any account.

Responses: `204 No Content`, `401 Unauthorized`, `500 Internal Server Error`

## Database Schema

### `users`

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `id` | `INT` | `PRIMARY KEY AUTO_INCREMENT` | Unique user identifier |
| `name` | `VARCHAR(255)` | `NOT NULL` | Display name |
| `password` | `VARCHAR(255)` | `NOT NULL` | Argon2 password hash (salt embedded in hash string) |
| `email` | `VARCHAR(255)` | `NOT NULL UNIQUE` | Email address, used for login |
| `is_admin` | `BOOLEAN` | `DEFAULT FALSE` | Admin flag; grants permission to delete other users |
| `created_at` | `TIMESTAMP` | `DEFAULT CURRENT_TIMESTAMP` | Account creation time |
| `modified_at` | `TIMESTAMP` | `DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP` | Last modification time |

**Notes:**

- Passwords are hashed with [Argon2id](https://en.wikipedia.org/wiki/Argon2) via the `argon2` crate. The salt is generated per-user and embedded in the hash string — no separate salt column is needed.
- There is no seed admin account. To promote the first admin after initial setup, sign up via the API then run:

  ```sql
  UPDATE users SET is_admin = TRUE WHERE email = 'your@email.com';
  ```

## Frontend Routes

| Path | Page |
| --- | --- |
| `/login` | Login form |
| `/signup` | Registration form |
| `/home` | File browser (authenticated) |
| `/profile` | User dashboard — storage stats, account info, quick actions |

## TypeScript Type Bindings

Types in [`frontend/vue-project/src/types/bindings/`](frontend/vue-project/src/types/bindings/) are auto-generated from the Rust backend structs using [ts-rs](https://github.com/Aleph-Alpha/ts-rs). **Do not edit these files manually.**

To regenerate after changing backend types:

```bash
cd backend
cargo run --features "export_binding"
```

## Project Structure

```text
.
├── backend/                  # Rust + Rocket API
│   └── src/
│       ├── main.rs
│       ├── routes/           # login, signup handlers
│       └── data_definitions/ # models, JWT, email
├── frontend/
│   └── vue-project/src/
│       ├── views/            # LoginPage, SignUp, HomePage, Profile
│       ├── components/ui/    # BaseBadge, BaseSpinner, BaseNotification,
│       │                     # NotAuthorized, UploadButton, BaseFileDescriptor
│       ├── stores/           # Pinia auth store
│       ├── router/           # Vue Router
│       └── types/
│           └── bindings/     # auto-generated from Rust
├── reverse_proxy/            # Nginx config + SSL
├── infra/                    # docker-compose.yml
├── .github/workflows/        # CI pipeline (backend + frontend tests)
└── .devcontainer/            # VS Code dev container configs
    ├── backend/
    └── frontend/
```

## CI

GitHub Actions runs on every push and pull request to `main`.

| Job | What it does |
| --- | --- |
| **Backend** | Spins up a MariaDB service container, applies the schema, and runs all tests including integration tests (email feature disabled) |
| **Frontend** | TypeScript type check + Vitest unit tests |
