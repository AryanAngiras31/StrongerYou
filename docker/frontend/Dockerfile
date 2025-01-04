# Use a full Node.js image for better compatibility
FROM node:18

# Set environment variables to avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    python3 \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Yarn globally
RUN npm install -g yarn

# Set the working directory
WORKDIR /usr/src/app

# Copy package.json and yarn.lock
COPY frontend/package*.json ./

# Install a specific version of Expo CLI globally
RUN yarn global add expo-cli@6.3.5

# Install project dependencies
RUN yarn install

# Copy the rest of the application code
COPY frontend/ .

# Expose Expo development ports
EXPOSE 19000 19001 19002

# Start the Expo development server
CMD ["expo", "start", "--tunnel"]
