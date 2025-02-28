FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Update Cargo to the latest version
RUN rustup update stable

# Build the dependencies
RUN cargo build --release

# Copy the rest of the application code
COPY . .

# Build the application
RUN cargo build --release

# Start the application
CMD ["./target/release/your_backend_executable"]