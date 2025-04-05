# Stage 1: Build the application
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Copy the source code
COPY backend/src ./src

# Set the DATABASE_URL environment variable
ENV DATABASE_URL=postgres://postgres:postgres@database:5432/strongeryou

# Set Cargo environment variables to increase network timeouts and retries
ENV CARGO_NET_RETRY=5
ENV CARGO_NET_TIMEOUT=600
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Update Cargo to the latest version
RUN rustup update stable

# Install SQLx CLI
RUN cargo install sqlx-cli

# Build the application
RUN cargo build --release

# Stage 2: Prepare the SQLx query cache
FROM rust:latest as sqlx-preparer

# Set the working directory
WORKDIR /usr/src/app

# Copy the built application from the builder stage
COPY --from=builder /usr/src/app/target/release/backend ./target/release/backend

# Copy the source code
COPY backend/src ./src

# Set the DATABASE_URL environment variable
ENV DATABASE_URL=postgres://postgres:postgres@database:5432/strongeryou

# Install SQLx CLI
RUN cargo install sqlx-cli

# Prepare the SQLx query cache
RUN cargo sqlx prepare -- --lib

# Stage 3: Create the final image
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the built application from the builder stage
COPY --from=builder /usr/src/app/target/release/backend ./target/release/backend

# Copy the prepared SQLx query cache from the sqlx-preparer stage
COPY --from=sqlx-preparer /usr/src/app/sqlx-data.json ./sqlx-data.json

# Copy the source code
COPY backend/src ./src

# Set the DATABASE_URL environment variable
ENV DATABASE_URL=postgres://postgres:postgres@database:5432/strongeryou

# Start the application
CMD ["./target/release/backend"]