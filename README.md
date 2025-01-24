# Template Axum

A Rust web server built using the **Axum** framework, leveraging powerful features like tracing, multipart handling, WebSocket support, and integration with a PostgreSQL database. This project is containerized and ready for deployment using a minimal Docker image.

## Features

- **Web Framework:** Axum with tracing, multipart, and WebSocket support.
- **Database:** SQLx for PostgreSQL with support for TLS, UUIDs, and chrono.
- **Background Tasks:** Scheduled tasks powered by `tokio-cron-scheduler`.
- **Authentication:** JSON Web Tokens (JWT) with bcrypt for password hashing.
- **Caching:** Redis connection pooling with `bb8`.
- **Utilities:** Includes Serde for JSON (de)serialization, regex utilities, and error handling using `thiserror` and `anyhow`.
- **Dockerized:** Optimized for production using a multi-stage build process with `distroless` base image.

## Getting Started

### Prerequisites

- Rust (https://www.rust-lang.org/)
- Docker (https://www.docker.com/)
- PostgreSQL (https://www.postgresql.org/)
- Redis (https://redis.io/)

### Setting Up

1. Clone the repository:

   ```bash
   git clone <repository_url>
   cd template_axum
   ```

2. Set up your `.env` file:

   Create a `.env` file in the root directory with the following variables:

   ```env
   DATABASE_URL=postgres://username:password@localhost:5432/database_name
   REDIS_URL=redis://localhost
   JWT_SECRET=your_secret_key
   ```

3. Run the application in development:

   ```bash
   cargo run
   ```

4. To run with Docker:

   Build and run the container:

   ```bash
   docker build -t template_axum .
   docker run -p 8080:8080 --env-file .env template_axum
   ```

### Scheduled Tasks

The project includes support for scheduled tasks using `tokio-cron-scheduler`. You can define your tasks in the `main.rs` or separate them into a module.

## Dependencies

- **Axum**: Web framework for async programming.
- **SQLx**: Async PostgreSQL database interface.
- **Tokio**: Async runtime.
- **Redis & bb8**: Caching and connection pooling.
- **JWT**: Authentication.
- **Tracing**: Observability and debugging.
- **Serde**: JSON serialization and deserialization.

## Building and Running

### Development

Run the application directly using Cargo:

```bash
cargo run
```

### Production

Use the provided Dockerfile to build an optimized production container:

```bash
docker build -t template_axum .
docker run -p 8080:8080 --env-file .env template_axum
```
