# Use a lightweight Node.js image
FROM node:18-slim

# Set the working directory
WORKDIR /usr/src/app

# Install dependencies
COPY frontend/package*.json ./
RUN npm set registry https://registry.npmjs.org/ && \
    npm set fetch-retries 5 && \
    npm set fetch-retry-mintimeout 20000 && \
    npm set fetch-retry-maxtimeout 120000 && \
    npm install

# Install Ionic CLI globally
RUN npm install -g @ionic/cli

# Copy the rest of the application code
COPY . .

# Start the Ionic application
CMD ["ionic", "serve", "--host", "0.0.0.0"]
