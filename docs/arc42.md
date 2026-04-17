# ARC42 Architekturdokumentation: Cloud Storage Platform

**Version:** 1.0  
**Datum:** 17.04.2026  
**Autor:** Julius Korbjuhn

---

## 1. Einführung und Ziele

### 1.1 Aufgabenstellung

Die Cloud Storage Platform ist eine selbst gehostete, private Cloud-Lösung, die es Benutzern ermöglicht, Dateien sicher zu speichern und zu verwalten – vergleichbar mit Nextcloud oder ownCloud, aber deutlich schlanker.

**Kernfunktionen:**
- Benutzerregistrierung und -authentifizierung (inkl. E-Mail-Bestätigung)
- Datei-Upload und Verwaltung über ein virtuelles Dateisystem
- Benutzerverwaltung mit Admin-Rollen
- Vollständig selbst gehostet (kein Vendor-Lock-in)

### 1.2 Qualitätsziele

| Priorität | Qualitätsmerkmal | Szenario |
|-----------|-----------------|----------|
| 1 | Sicherheit | Passwörter via Argon2id, JWT in HttpOnly-Cookies, Rate Limiting |
| 2 | Zuverlässigkeit | Transaktionale Operationen, Health-Checks, Retry-Logik |
| 3 | Wartbarkeit | Rust-Typsystem, automatisch generierte TypeScript-Bindings, Repository-Pattern |
| 4 | Portabilität | Vollständig containerisiert via Docker Compose |

### 1.3 Stakeholder

| Rolle | Erwartung |
|-------|-----------|
| Endbenutzer | Dateien sicher hochladen und verwalten |
| Administrator | Benutzer verwalten, System überwachen |
| Entwickler | Klare Codestruktur, lokale Entwicklungsumgebung (DevContainer) |

---

## 2. Randbedingungen

### 2.1 Technische Randbedingungen

| Randbedingung | Hintergrund |
|---------------|-------------|
| Rust für das Backend | Speichersicherheit, Performance, starkes Typsystem |
| Docker als Deployment-Einheit | Reproduzierbare Umgebungen, einfaches Self-Hosting |
| MariaDB als Datenbank | ACID-Transaktionen, geringe Ressourcenanforderungen |
| MinIO für Objektspeicher | S3-kompatibel, selbst gehostet, kein Vendor-Lock-in |
| HTTPS-Pflicht | Sicherheitsanforderung für JWT-Cookies (`Secure`-Flag) |

### 2.2 Organisatorische Randbedingungen

- **Lizenz:** Community Use License (nicht redistributierbar)
- **Repository:** https://github.com/FalkAurel/Cloud
- **CI/CD:** GitHub Actions (Tests, Type-Checking)
- **Node.js:** ≥ 20.19.0 oder ≥ 22.12.0

---

## 3. Kontextabgrenzung

### 3.1 Fachlicher Kontext

```
┌─────────────────────────────────────────────────────────────┐
│                    Cloud Storage Platform                   │
│                                                             │
│  ┌─────────────┐    ┌───────────────┐    ┌───────────────┐  │
│  │  Frontend   │    │   Backend     │    │   Speicher    │  │
│  │  (Vue 3)    │◄──►│   (Rocket)   │◄──►│ MariaDB/MinIO │  │
│  └─────────────┘    └───────┬───────┘    └───────────────┘  │
│                             │                               │
└─────────────────────────────┼───────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   SMTP-Server     │
                    │ (E-Mail-Versand)  │
                    └───────────────────┘
```

| Nachbarsystem | Richtung | Beschreibung |
|---------------|----------|-------------|
| Browser/Client | ↔ | Endbenutzer-Interaktion via HTTPS |
| SMTP-Server (Gmail) | → | Transaktionale E-Mails (Registrierungsbestätigung) |

### 3.2 Technischer Kontext

| Schnittstelle | Protokoll | Format |
|--------------|-----------|--------|
| Frontend ↔ Nginx | HTTPS | REST-JSON, Datei-Streams |
| Nginx → Backend | HTTP | Reverse Proxy |
| Nginx → Frontend | HTTP | Statische Dateien |
| Backend ↔ MariaDB | MySQL-Protokoll (SQLx) | SQL |
| Backend ↔ MinIO | S3 API (HTTP) | Binär-Chunks (8 MB) |
| Backend → SMTP | SMTP-TLS | E-Mail (RFC 5321) |

---

## 4. Lösungsstrategie

| Entscheidung | Rationale |
|--------------|-----------|
| **Rust + Rocket** als Backend | Speichersicherheit, Zero-Cost-Abstraktionen, starkes Typsystem verhindert ganze Fehlerklassen |
| **Vue 3 + TypeScript** als Frontend | Reaktives Daten-Binding, Composition API, kleineres Bundle als React/Angular |
| **JWT in HttpOnly-Cookies** | Schutz vor XSS-Angriffen (kein JS-Zugriff auf Token) |
| **Argon2id** für Passwort-Hashing | Speicher-hart, widerstandsfähig gegen GPU-Angriffe |
| **MinIO** für Objektspeicher | S3-kompatibel, selbst gehostet, effizient für beliebige Dateigrößen |
| **ts-rs** für TypeScript-Bindings | Typen aus Rust automatisch generieren → Frontend/Backend immer synchron |
| **Chunked Uploads (8 MB)** | Minimaler Speicher-Footprint bei großen Dateien |
| **Timing-Attack-Resistenz** beim Login | Dummy-Hash bei unbekannten E-Mails verhindert User-Enumeration |

---

## 5. Bausteinsicht

### 5.1 Ebene 1: Gesamtsystem

```
┌─────────────────────────────────────────────────────────────────┐
│                        Cloud Platform                           │
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐  │
│  │ reverse_proxy │  │   frontend    │  │      backend        │  │
│  │   (Nginx)     │  │   (Vue 3)     │  │  (Rust/Rocket)      │  │
│  └───────┬───────┘  └───────────────┘  └──────────┬──────────┘  │
│          │                                        │             │
│          │          ┌───────────────┐  ┌──────────▼──────────┐  │
│          │          │   MariaDB     │  │        MinIO        │  │
│          │          │  (Relationale │  │  (Objektspeicher)   │  │
│          │          │   Daten)      │  │                     │  │
│          │          └───────────────┘  └─────────────────────┘  │
└──────────┼──────────────────────────────────────────────────────┘
           │
       Internet
```

### 5.2 Ebene 2: Backend-Bausteine

| Baustein | Verantwortlichkeit |
|----------|--------------------|
| **routes/** | HTTP-Request-Handler, Eingabevalidierung, Response-Aufbau |
| **database/user_repository** | CRUD-Operationen für Benutzer |
| **database/virtual_filesystem** | Datei-Metadaten, Verzeichnisstruktur |
| **object_storage/** | Abstraktion über MinIO (S3-API) |
| **data_definitions/** | Domain-Typen: `StandardUserView`, JWT-Claims, E-Mail-Konfiguration |

#### Backend-Bausteine im Detail

**`routes/`**

| Modul | Route | Methode | Beschreibung |
|-------|-------|---------|-------------|
| `signup.rs` | `/v1/signup` | POST | Registrierung mit E-Mail-Bestätigung |
| `login.rs` | `/v1/login` | POST | Authentifizierung, JWT-Cookie setzen |
| `logout.rs` | `/v1/logout` | POST | JWT-Cookie löschen |
| `me.rs` | `/v1/me` | GET | Aktuelles Benutzerprofil |
| `delete_user.rs` | `/v1/users/:id` | DELETE | Benutzer löschen (selbst oder Admin) |
| `upload.rs` | `/v1/upload` | POST | Datei-Upload zu MinIO |

**`data_definitions/`**

| Modul | Beschreibung |
|-------|-------------|
| `user.rs` | `StandardUserView`, `UserLoginRequest`, `UserSignupRequest` |
| `jwt.rs` | JWT-Erstellung/Verifikation (HMAC-SHA256, ms-Präzision) |
| `email.rs` | SMTP-Pool, E-Mail-Komposition, Retry-Logik |
| `fixed_len_str.rs` | UTF-8-String mit max. 40 Zeichen (160 Byte) |

### 5.3 Ebene 2: Frontend-Bausteine

| Baustein | Verantwortlichkeit |
|----------|--------------------|
| **views/** | Seiten: Login, Registrierung, Home (Datei-Browser), Profil |
| **components/** | Wiederverwendbare UI: BaseInput, BaseButton, UploadButton, FileDescriptor |
| **stores/auth.ts** | Zentraler Auth-Zustand via Pinia (`isAuthenticated`, `user`) |
| **router/** | Routen-Guards prüfen JWT via `GET /me` |
| **types/bindings/** | Auto-generierte TypeScript-Typen aus Rust-Structs |

---

## 6. Laufzeitsicht

### 6.1 Szenario: Benutzer-Registrierung

```
Browser          Nginx          Backend           MariaDB         SMTP
  │                │                │                │              │
  │──POST /signup──►                │                │              │
  │                │──POST /v1/signup►               │              │
  │                │                │─Validierung    │              │
  │                │                │─E-Mail-Check──►│              │
  │                │                │◄───────────────│              │
  │                │                │─Argon2id-Hash  │              │
  │                │                │─INSERT user───►│              │
  │                │                │◄───────────────│              │
  │                │                │─Bestätigungs-E-Mail──────────►│
  │                │                │◄──────────────────────────────│
  │◄───────────────────201 Created──│                │              │
```

### 6.2 Szenario: Benutzer-Login

```
Browser          Nginx          Backend           MariaDB
  │                │                │                │
  │──POST /login───►                │                │
  │                │──POST /v1/login►               │
  │                │                │─SELECT by email►│
  │                │                │◄───────────────│
  │                │                │─Argon2id verify│
  │                │                │─JWT generieren │
  │◄───200 OK + JWT-Cookie──────────│                │
```

### 6.3 Szenario: Datei-Upload

```
Browser          Nginx          Backend           MariaDB         MinIO
  │                │                │                │              │
  │──POST /upload──►                │                │              │
  │  (Headers: Content-Length,      │                │              │
  │   X-Filename, X-IsFolder)       │                │              │
  │                │──POST /v1/upload►               │              │
  │                │                │─JWT verifizieren│             │
  │                │                │─Stream → MinIO─────────────►  │
  │                │                │  (8 MB Chunks) │              │
  │                │                │◄───────────────────────────── │
  │                │                │─INSERT file───►│              │
  │                │                │◄───────────────│              │
  │◄───────────────────200 OK───────│                │              │
```

### 6.4 Szenario: JWT-Ablauf-Prüfung (Frontend)

```
Browser          Router Guard     Backend
  │                │                │
  │─Navigation─────►                │
  │                │──GET /v1/me───►│
  │                │                │─JWT prüfen
  │                │◄──401 Unauth───│ (abgelaufen)
  │◄──Redirect /login───────────────│
```

---

## 7. Verteilungssicht

### 7.1 Infrastruktur (Docker Compose)

```
┌────────────────────────────────────────────────────────┐
│                    Docker Host                         │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │               internal_network (bridge)          │  │
│  │                                                  │  │
│  │  ┌────────────┐      ┌──────────┐                │  │
│  │  │   Nginx    │:80   │ Frontend │:80             │  │
│  │  │ :80,:443   │─────►│ (Vue 3)  │                │  │
│  │  │            │      └──────────┘                │  │
│  │  │            │:3000 ┌──────────┐                │  │
│  │  │            │─────►│ Backend  │                │  │
│  │  └────────────┘      │ (Rocket) │:3000           │  │
│  │      ▲ ▲             └──┬───┬───┘                │  │
│  │      │ │                │   │                    │  │
│  │  :80 │ │:443           │   │                     │  │
│  │      │ │          :3306│   │:9000                │  │
│  │      │ │     ┌─────────▼┐  ▼──────────┐          │  │
│  │      │ │     │  MariaDB │  │  MinIO    │         │  │
│  │      │ │     │  :3306   │  │  :9000    │         │  │
│  │      │ │     └──────────┘  │  :9001    │         │  │
│  │      │ │         │         └───────────┘         │  │
│  │      │ │      db_data       minio_data           │  │
│  │      │ │     (Volume)       (Volume)             │  │
│  └──────┼─┼─────────────────────────────────────────|  │
│         │ │                                         │  │
└─────────┼─┼─────────────────────────────────────────┘──|
          │ │
     Port 80 Port 443
      (Internet)
```

### 7.2 Container-Konfiguration

| Container | Image | Ports | Volumes | Health Check |
|-----------|-------|-------|---------|-------------|
| `reverse_proxy` | nginx:stable-alpine | 80, 443 | SSL-Certs | - |
| `frontend` | Vue 3 Vite-Build | intern:80 | - | - |
| `backend` | Rust scratch-Image | intern:3000 | .env | - |
| `db` | mariadb:12.2 | intern:3306 | db_data | mariadb-admin ping |
| `minio` | minio/minio | intern:9000, 9001 | minio_data | curl /minio/health/live |

### 7.3 DevContainer-Setup

Zwei separate VS Code DevContainer-Konfigurationen:

| DevContainer | Services | Ports | Extensions |
|-------------|----------|-------|------------|
| Backend | backend, db, minio | 8000:3000, 9001:9001 | rust-analyzer, LLDB |
| Frontend | frontend | 5173 | Volar, Prettier |

---

## 8. Querschnittskonzepte

### 8.1 Sicherheit

| Konzept | Implementierung |
|---------|----------------|
| **Passwort-Hashing** | Argon2id (speicher-hart, per-User-Salt, eingebettet im Hash) |
| **Authentifizierung** | JWT (HMAC-SHA256/HS256), 10 Minuten Ablaufzeit |
| **Session-Management** | JWT in `HttpOnly`, `Secure`, `SameSite=Lax` Cookie |
| **Timing-Attacken** | Dummy-Hash bei unbekannter E-Mail → verhindert User-Enumeration |
| **Rate Limiting** | Nginx: 5 req/min auf `/api/login`, `/api/signup` (Burst: 3) |
| **Transport-Sicherheit** | HTTPS (TLS), HSTS-Header (63 Tage) |
| **Autorisierung** | `is_admin`-Boolean, Rollenprüfung im Request-Handler |
| **XSS-Schutz** | `X-Content-Type-Options`, `X-Frame-Options: DENY`, CSP-Header |

### 8.2 Persistenz

| Datenkategorie | Speicherort | Mechanismus |
|----------------|------------|-------------|
| Benutzerdaten | MariaDB (`users`-Tabelle) | SQLx async, Connection-Pool |
| Datei-Metadaten | MariaDB (`files`-Tabelle) | UUID-Primärschlüssel, Parent-Hierarchie |
| Datei-Inhalte | MinIO | S3-API, 8 MB Chunks, UUID-basierte Namen |
| Gruppen | MariaDB (`user_groups`, `group_members`) | Noch nicht in Benutzung |

### 8.3 Fehlerbehandlung

| Schicht | Strategie |
|---------|----------|
| Backend-Routes | HTTP-Statuscodes (201, 204, 400, 401, 409, 500) |
| Datenbank | SQLx-Fehlertypen, Transaktions-Rollback |
| E-Mail-Versand | Retry-Logik (3 Versuche, 30s Wartezeit); bei Fehler → User-Rollback |
| MinIO-Upload | Fehler-Response, aber **kein automatisches Rollback** bei DB-Fehler danach |
| Frontend | Pinia-Zustand, BaseNotification-Komponente (Toast-ähnlich) |

### 8.4 Logging & Observability

| Komponente | Mechanismus |
|------------|------------|
| Backend | `tracing` + `tracing-subscriber` (strukturiertes Logging) |
| Nginx | Access-Logs (Standard) |
| Health Check | `GET /health` → 200 "Healthy" |

### 8.5 Typsicherheit Frontend/Backend

Rust-Structs werden via `ts-rs` automatisch in TypeScript-Typen kompiliert:

```
Rust (data_definitions/user.rs)
    └── StandardUserView, UserLoginRequest, ...
         └── [ts-rs compile step]
              └── frontend/src/types/bindings/*.ts
```

Dies verhindert Typ-Divergenzen zwischen API-Contract und Frontend-Code.

### 8.6 API-Versionierung

Alle Endpunkte unter dem Präfix `/v1/` für Rückwärtskompatibilität.

---

## 9. Architekturentscheidungen

### ADR-001: JWT in HttpOnly-Cookies statt localStorage

**Kontext:** Session-Management für SPA  
**Entscheidung:** JWT wird im HttpOnly-Cookie gespeichert  
**Begründung:** Schutz vor XSS – kein JavaScript-Zugriff auf den Token möglich  
**Konsequenz:** CSRF-Risiko (mitigiert durch `SameSite=Lax`)

---

### ADR-002: Argon2id für Passwort-Hashing

**Kontext:** Wahl des Hashing-Algorithmus  
**Entscheidung:** Argon2id (Gewinner des Password Hashing Competition 2015)  
**Begründung:** Speicher-hart und CPU-intensiv → widerstandsfähig gegen GPU-Brute-Force  
**Konsequenz:** Höhere CPU-Last bei Login/Signup

---

### ADR-003: MinIO als Objektspeicher

**Kontext:** Speicherung beliebiger Dateien  
**Entscheidung:** MinIO (S3-kompatibel)  
**Begründung:** Kein Vendor-Lock-in, selbst gehostet, S3-API ermöglicht spätere Migration  
**Konsequenz:** Zusätzlicher Service im Stack, Netzwerk-Overhead für Dateitransfers

---

### ADR-004: Kein Refresh-Token-Mechanismus

**Kontext:** JWT-Ablauf nach 10 Minuten  
**Entscheidung:** Kein Refresh-Token, User muss sich nach Ablauf neu einloggen  
**Begründung:** Reduzierte Komplexität, ausreichend für typische Nutzungsszenarien  
**Konsequenz:** Schlechtere UX bei langen Sessions

---

### ADR-005: Timing-Attack-Resistenz beim Login

**Kontext:** Schutz vor User-Enumeration  
**Entscheidung:** Bei unbekannter E-Mail wird DUMMY_HASH-Konstante geprüft  
**Begründung:** Ohne Dummy-Hash antwortet der Server schneller → Angreifer kann existierende Accounts identifizieren  
**Konsequenz:** Marginaler Performance-Overhead bei fehlgeschlagenen Logins

---

## 10. Qualitätsszenarien

| Qualitätsmerkmal | Szenario | Maßnahme | Ergebnis |
|-----------------|----------|----------|---------|
| **Sicherheit** | Angreifer versucht User-Enumeration via Login-Timing | Dummy-Hash bei unbekannter E-Mail | Keine zeitliche Unterscheidung möglich |
| **Sicherheit** | Brute-Force auf Login | Rate Limiting (5 req/min, Nginx) | Angriff verlangsamt |
| **Sicherheit** | XSS-Angriff auf JWT | HttpOnly-Cookie | Kein JS-Zugriff auf Token |
| **Zuverlässigkeit** | E-Mail-Versand schlägt fehl | 3 Retry-Versuche, dann Rollback | Keine inkonsistenten Zustände |
| **Wartbarkeit** | TypeScript-Typ stimmt nicht mit API überein | ts-rs generiert Typen aus Rust | Compile-Zeit-Fehler statt Runtime-Fehler |
| **Portabilität** | Deployment auf neuem Server | Docker Compose + .env | Einzeilige Deployment-Anweisung |

---

## 11. Risiken und technische Schulden

| Risiko / Schuld | Schwere | Beschreibung | Empfehlung |
|----------------|---------|-------------|------------|
| **Orphaned MinIO-Objekte** | Mittel | Bei DB-Fehler nach erfolgreichem MinIO-Upload entstehen verwaiste Objekte | Sagas/Kompensations-Transaktionen oder MinIO-Lifecycle-Policies |
| **Kein Datei-Download** | Hoch | Upload-Endpunkt existiert, Download fehlt | `GET /v1/files/:id` mit MinIO Presigned URLs implementieren |
| **Kein Datei-Löschen** | Hoch | Dateien können nicht gelöscht werden | `DELETE /v1/files/:id` inkl. MinIO-Objekt-Löschung |
| **Gruppen nicht implementiert** | Niedrig | Schema existiert, Logik fehlt | Sharing-Funktionalität auf Basis der Gruppenstruktur aufbauen |
| **Kein Refresh-Token** | Mittel | 10-Minuten-Sessions erfordern häufige Re-Authentifizierung | Refresh-Token-Flow implementieren |
| **Self-Signed Zertifikat** | Mittel | Browser-Warnungen in Produktion | Let's Encrypt / ACME-Integration in Nginx |
| **IP-basiertes Rate Limiting** | Niedrig | Funktioniert nicht hinter Load Balancer | `X-Forwarded-For`-Header in Nginx konfigurieren |
| **Keine Datei-Downloadbeschränkung** | Niedrig | Kein Speicherlimit pro Benutzer | User-Quota in DB und Upload-Validierung |

---

## 12. Glossar

| Begriff | Definition |
|---------|-----------|
| **Argon2id** | Speicher-hartés Passwort-Hashing-Verfahren, Gewinner des PHC 2015 |
| **JWT** | JSON Web Token – stateless Authentifizierungs-Token |
| **MinIO** | S3-kompatibler, selbst gehosteter Objektspeicher |
| **Rocket** | Asynchrones Web-Framework für Rust |
| **SQLx** | Async Rust-Datenbank-Treiber mit Compile-Zeit-verifizierten Queries |
| **ts-rs** | Rust-Crate zur automatischen TypeScript-Typgenerierung aus Rust-Structs |
| **Pinia** | State-Management-Bibliothek für Vue 3 |
| **Request Guard** | Rocket-Mechanismus für Middleware (z. B. JWT-Verifikation) |
| **Virtual Filesystem** | Datei-Metadaten mit Parent-Kind-Beziehungen in der DB (nicht POSIX) |
| **DevContainer** | VS Code Remote Container-Konfiguration für reproduzierbare Entwicklungsumgebungen |
| **HSTS** | HTTP Strict Transport Security – erzwingt HTTPS-Nutzung |
| **ADR** | Architecture Decision Record – dokumentiert eine Architekturentscheidung |
