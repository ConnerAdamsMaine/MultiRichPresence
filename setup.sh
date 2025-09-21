#!/bin/bash

echo "Discord Rich Presence App Setup"
echo "=================================="
echo

# Check if Rust is installed
echo "Step 1: Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust is not installed!"
    echo "Please install Rust from https://rustup.rs/"
    echo "Then run this script again."
    exit 1
fi
echo "✓ Rust is installed"
echo

# Build the application
echo "Step 2: Building the application..."
echo "This may take a few minutes on first run..."
if ! cargo build --release; then
    echo "ERROR: Build failed!"
    echo "Check the error messages above."
    exit 1
fi
echo "✓ Build completed successfully"
echo

# Setup instructions
echo "Step 3: Setup instructions"
echo "=========================="
echo
echo "Before running the app, you need to:"
echo "1. Go to https://discord.com/developers/applications"
echo "2. Create a new application"
echo "3. Copy the Application ID"
echo "4. Replace APP_ID in main.rs with your Application ID"
echo "5. Rebuild with: cargo build --release"
echo
echo "The executable is located at: target/release/discord-rich-presence-app"
echo

# Ask if user wants to run the app
read -p "Do you want to run the app now? (y/n): " choice
case "$choice" in 
    y|Y|yes|YES ) 
        echo
        echo "Starting Discord Rich Presence App..."
        ./target/release/discord-rich-presence-app
        ;;
    * ) 
        echo
        echo "You can run the app later by executing:"
        echo "./target/release/discord-rich-presence-app"
        ;;
esac

echo
echo "Setup complete! Don't forget to configure your Discord Application ID."

# Make the script executable for future use
chmod +x "$0"