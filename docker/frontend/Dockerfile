# Use a lightweight Node.js image
FROM node:18-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy package.json and install dependencies
COPY frontend/package*.json ./
RUN npm install -g expo-cli 

# Copy the application code
COPY frontend/ .

# Expose Expo development port
EXPOSE 19000 19001 19002

# Start the Expo development server
CMD ["expo", "start", "--tunnel"]
