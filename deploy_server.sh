#!/bin/bash

# Rune VCS Home Server Deployment Script
# This script sets up a Rune VCS server anywhere (home, cloud, VPS, etc.)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_NAME="${1:-rune-server}"
PORT="${2:-8080}"
DOMAIN="${3:-localhost}"

echo "🚀 Setting up Rune VCS Server: $SERVER_NAME"
echo "📍 Domain: $DOMAIN"
echo "🔌 Port: $PORT"

# Check dependencies
check_dependencies() {
    echo "🔍 Checking dependencies..."
    
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker is required but not installed"
        echo "Please install Docker: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        echo "❌ Docker Compose is required but not installed"
        echo "Please install Docker Compose: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    echo "✅ Dependencies check passed"
}

# Generate server configuration
setup_config() {
    echo "⚙️ Setting up server configuration..."
    
    # Create server directory
    SERVER_DIR="$HOME/rune-servers/$SERVER_NAME"
    mkdir -p "$SERVER_DIR"
    cd "$SERVER_DIR"
    
    # Generate authentication token
    AUTH_TOKEN=$(openssl rand -hex 32)
    
    # Create environment file
    cat > .env << EOF
# Rune VCS Server Configuration
RUNE_SERVER_NAME=$SERVER_NAME
RUNE_PORT=$PORT
RUNE_DOMAIN=$DOMAIN
RUNE_AUTH_TOKEN=$AUTH_TOKEN
RUNE_DATA_DIR=./data
RUNE_LOG_LEVEL=info

# Database (for multi-user setups)
POSTGRES_DB=rune
POSTGRES_USER=rune
POSTGRES_PASSWORD=$(openssl rand -base64 32)

# SSL (set to true for production)
RUNE_SSL_ENABLED=false
RUNE_SSL_CERT_PATH=
RUNE_SSL_KEY_PATH=
EOF

    echo "✅ Configuration created in $SERVER_DIR"
    echo "🔑 Authentication token: $AUTH_TOKEN"
    echo ""
    echo "📁 Server directory: $SERVER_DIR"
    echo "📋 Save this token - you'll need it to connect clients!"
}

# Setup Docker Compose
setup_docker() {
    echo "🐳 Setting up Docker environment..."
    
    # Copy docker-compose file
    cp "$SCRIPT_DIR/docker-compose.multi-server.yml" ./docker-compose.yml
    
    # Create data directories
    mkdir -p data/repositories data/lfs data/logs
    
    # Set permissions
    chmod 755 data/repositories data/lfs data/logs
    
    echo "✅ Docker environment ready"
}

# Start server
start_server() {
    echo "🚀 Starting Rune VCS server..."
    
    # Pull latest images
    docker-compose pull
    
    # Start services
    docker-compose up -d
    
    # Wait for health check
    echo "⏳ Waiting for server to be ready..."
    sleep 10
    
    # Check health
    if curl -f "http://localhost:$PORT/health" &> /dev/null; then
        echo "✅ Server is running and healthy!"
        echo ""
        echo "🌐 Server URL: http://$DOMAIN:$PORT"
        echo "🔑 Auth Token: $(grep RUNE_AUTH_TOKEN .env | cut -d= -f2)"
        echo ""
        echo "📖 Next steps:"
        echo "  1. Configure your client: rune remote add origin http://$DOMAIN:$PORT --token YOUR_TOKEN"
        echo "  2. Clone repositories: rune clone http://$DOMAIN:$PORT/repo-name ./local-repo"
        echo "  3. View logs: docker-compose logs -f"
        echo "  4. Stop server: docker-compose down"
        echo ""
        echo "📚 Full guide: https://github.com/CaptainOtto/rune-vcs/blob/main/HOME_SERVER_GUIDE.md"
    else
        echo "❌ Server failed to start properly"
        echo "🔍 Check logs with: docker-compose logs"
        exit 1
    fi
}

# Main execution
main() {
    echo "🏠 Rune VCS Home Server Setup"
    echo "=================================="
    
    check_dependencies
    setup_config
    setup_docker
    start_server
    
    echo ""
    echo "🎉 Rune VCS Server setup complete!"
    echo "📂 Server location: $SERVER_DIR"
}

# Handle script options
case "${1:-}" in
    --help|-h)
        echo "Rune VCS Server Deployment Script"
        echo ""
        echo "Usage: $0 [server-name] [port] [domain]"
        echo ""
        echo "Arguments:"
        echo "  server-name  Name for the server instance (default: rune-server)"
        echo "  port         Port to run on (default: 8080)"
        echo "  domain       Domain or IP address (default: localhost)"
        echo ""
        echo "Examples:"
        echo "  $0                           # Basic local setup"
        echo "  $0 my-home-server 9000       # Custom name and port"
        echo "  $0 prod-server 80 example.com # Production setup"
        echo ""
        echo "Environment Variables:"
        echo "  RUNE_SSL_ENABLED=true        # Enable SSL"
        echo "  RUNE_LOG_LEVEL=debug         # Set log level"
        exit 0
        ;;
    *)
        main
        ;;
esac
