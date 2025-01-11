# Use the official Rust image for development
FROM rust:1.72-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the Rust backend source code
COPY backend/ .

# Install dependencies and compile the application in release mode
RUN cargo build

# Expose backend server port (e.g., 8080)
EXPOSE 8080

# Run the application
CMD ["cargo", "run"]

