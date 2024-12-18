# Use the official Flutter image
FROM ubuntu:latest

# Prevent prompts from packages
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    unzip \
    xz-utils \
    zip \
    libglu1-mesa \
    openjdk-17-jdk-headless

# Download and install Flutter
RUN git clone https://github.com/flutter/flutter.git /usr/local/flutter
ENV PATH="/usr/local/flutter/bin:${PATH}"

# Enable Flutter web support
RUN flutter channel stable
RUN flutter upgrade
RUN flutter config --enable-web

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Get Flutter dependencies
RUN flutter pub get

# Development mode setup
CMD ["flutter", "run", "--web-port=8000", "--web-hostname=0.0.0.0"]

# Expose the Flutter development server
EXPOSE 8000