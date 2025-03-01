# Stage 1: Build the application and prepare the SQLx query cache
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Copy the source code
COPY backend/src ./src

# Set the DATABASE_URL environment variable
ENV DATABASE_URL=postgres://postgres:postgres@database:5432/strongeryou

# Update Cargo to the latest version
RUN rustup update stable

# Install SQLx CLI
RUN cargo install sqlx-cli

# Prepare the SQLx query cache
RUN cargo sqlx prepare -- --lib

# Build the application
RUN cargo build --release

# Stage 2: Create the final image
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the built application from the builder stage
COPY --from=builder /usr/src/app/target/release/backend ./target/release/backend

# Copy the source code
COPY backend/src ./src

# Set the DATABASE_URL environment variable
ENV DATABASE_URL=postgres://postgres:postgres@database:5432/strongeryou

# Start the application
CMD ["./target/release/backend"]