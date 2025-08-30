#!/bin/bash

# Rune VCS Server Setup Script
# Universal server deployment for any environment
# Works like git for easy remote server setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default configuration
DEFAULT_PORT=8421
DEFAULT_LFS_PORT=8420
DEFAULT_DATA_DIR="/var/lib/rune"
DEFAULT_CONFIG_DIR="/etc/rune"
SERVER_TYPE="standalone"
DOMAIN=""
USE_SSL=false
ADMIN_TOKEN=""
BACKUP_ENABLED=true

show_help() {
    cat << EOF
🚀 Rune VCS Server Setup - Deploy Anywhere

USAGE:
    $0 [OPTIONS] COMMAND

COMMANDS:
    install         Install Rune server on this machine
    docker          Deploy using Docker (recommended)
    cluster         Set up multi-server cluster
    ssl             Configure SSL/TLS certificates
    backup          Configure automated backups
    test            Test server deployment

OPTIONS:
    -p, --port PORT         API server port (default: $DEFAULT_PORT)
    -l, --lfs-port PORT     LFS server port (default: $DEFAULT_LFS_PORT)
    -d, --data-dir DIR      Data directory (default: $DEFAULT_DATA_DIR)
    -c, --config-dir DIR    Config directory (default: $DEFAULT_CONFIG_DIR)
    -D, --domain DOMAIN     Server domain (for SSL)
    -s, --ssl               Enable SSL/TLS
    -t, --token TOKEN       Admin authentication token
    -b, --backup-dir DIR    Backup directory
    --no-backup            Disable automated backups
    -h, --help             Show this help

EXAMPLES:
    # Simple local server
    $0 install

    # Production server with SSL
    $0 -D git.mycompany.com -s -t "secure-token-123" install

    # Docker deployment
    $0 -p 80 -l 8080 -D git.example.com docker

    # Multi-server cluster
    $0 cluster

    # Test existing deployment
    $0 test

EOF
}

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check system requirements
check_requirements() {
    log "Checking system requirements..."
    
    # Check OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    else
        error "Unsupported operating system: $OSTYPE"
    fi
    
    # Check if running as root for system installation
    if [[ $EUID -eq 0 ]] && [[ "$1" != "docker" ]]; then
        warning "Running as root - will install system-wide"
    fi
    
    # Check Docker availability for Docker deployment
    if [[ "$1" == "docker" ]] && ! command -v docker &> /dev/null; then
        error "Docker is required for Docker deployment"
    fi
    
    success "System requirements check passed"
}

# Download Rune binary
download_rune() {
    log "Downloading Rune VCS binary..."
    
    local arch
    case $(uname -m) in
        x86_64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
    
    local platform
    case $OS in
        linux) platform="unknown-linux-gnu" ;;
        macos) platform="apple-darwin" ;;
    esac
    
    local download_url="https://github.com/CaptainOtto/rune-vcs/releases/latest/download/rune-${arch}-${platform}.tar.gz"
    local temp_dir=$(mktemp -d)
    
    curl -L "$download_url" -o "$temp_dir/rune.tar.gz" || error "Failed to download Rune binary"
    tar -xzf "$temp_dir/rune.tar.gz" -C "$temp_dir" || error "Failed to extract Rune binary"
    
    # Install binary
    if [[ $EUID -eq 0 ]]; then
        mv "$temp_dir/rune" /usr/local/bin/rune
        chmod +x /usr/local/bin/rune
    else
        mkdir -p "$HOME/.local/bin"
        mv "$temp_dir/rune" "$HOME/.local/bin/rune"
        chmod +x "$HOME/.local/bin/rune"
        export PATH="$HOME/.local/bin:$PATH"
    fi
    
    rm -rf "$temp_dir"
    success "Rune binary installed successfully"
}

# Create server configuration
create_server_config() {
    log "Creating server configuration..."
    
    local config_dir=$1
    local data_dir=$2
    local port=$3
    local lfs_port=$4
    
    mkdir -p "$config_dir"
    mkdir -p "$data_dir"
    mkdir -p "$data_dir/repositories"
    mkdir -p "$data_dir/lfs"
    mkdir -p "$data_dir/logs"
    
    cat > "$config_dir/server.yml" << EOF
# Rune VCS Server Configuration
server:
  host: "0.0.0.0"
  port: $port
  
lfs:
  host: "0.0.0.0" 
  port: $lfs_port
  storage_path: "$data_dir/lfs"

repositories:
  storage_path: "$data_dir/repositories"
  
authentication:
  admin_token: "$ADMIN_TOKEN"
  require_auth: true
  
logging:
  level: "info"
  file: "$data_dir/logs/rune-server.log"
  
monitoring:
  metrics_enabled: true
  health_check_enabled: true

backup:
  enabled: $BACKUP_ENABLED
  interval: "24h"
  retention: "30d"
  path: "$data_dir/backups"
EOF

    if [[ "$USE_SSL" == "true" && -n "$DOMAIN" ]]; then
        cat >> "$config_dir/server.yml" << EOF

ssl:
  enabled: true
  cert_file: "$config_dir/ssl/$DOMAIN.crt"
  key_file: "$config_dir/ssl/$DOMAIN.key"
  auto_cert: true
EOF
    fi
    
    success "Server configuration created at $config_dir/server.yml"
}

# Create systemd service (Linux only)
create_systemd_service() {
    if [[ "$OS" != "linux" ]] || [[ $EUID -ne 0 ]]; then
        return
    fi
    
    log "Creating systemd service..."
    
    cat > /etc/systemd/system/rune-server.service << EOF
[Unit]
Description=Rune VCS Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=rune
Group=rune
ExecStart=/usr/local/bin/rune api --config $DEFAULT_CONFIG_DIR/server.yml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    # Create rune user
    if ! id "rune" &>/dev/null; then
        useradd -r -s /bin/false -d "$DEFAULT_DATA_DIR" rune
    fi
    
    chown -R rune:rune "$DEFAULT_DATA_DIR"
    chown -R rune:rune "$DEFAULT_CONFIG_DIR"
    
    systemctl daemon-reload
    systemctl enable rune-server
    
    success "Systemd service created and enabled"
}

# Install standalone server
install_standalone() {
    log "Installing Rune VCS standalone server..."
    
    check_requirements "standalone"
    download_rune
    create_server_config "$DEFAULT_CONFIG_DIR" "$DEFAULT_DATA_DIR" "$DEFAULT_PORT" "$DEFAULT_LFS_PORT"
    create_systemd_service
    
    success "Standalone server installation complete!"
    echo ""
    echo "Next steps:"
    echo "1. Start the server: sudo systemctl start rune-server"
    echo "2. Check status: sudo systemctl status rune-server"
    echo "3. View logs: sudo journalctl -u rune-server -f"
    echo "4. Test: curl http://localhost:$DEFAULT_PORT/sync/info"
}

# Deploy with Docker
deploy_docker() {
    log "Deploying Rune VCS with Docker..."
    
    check_requirements "docker"
    
    # Create docker-compose.yml
    cat > docker-compose.yml << EOF
version: '3.8'

services:
  rune-server:
    image: rune-vcs:latest
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "$DEFAULT_PORT:8421"
      - "$DEFAULT_LFS_PORT:8420"
    environment:
      - RUNE_AUTH_TOKEN=$ADMIN_TOKEN
      - RUNE_DATA_DIR=/data
    volumes:
      - rune_data:/data
      - rune_logs:/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8421/sync/info"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - rune_ssl:/etc/ssl:ro
    depends_on:
      - rune-server
    restart: unless-stopped

volumes:
  rune_data:
  rune_logs:
  rune_ssl:

networks:
  default:
    name: rune-network
EOF

    # Build and start
    docker-compose up -d
    
    success "Docker deployment complete!"
    echo ""
    echo "Services running:"
    echo "- Rune API: http://localhost:$DEFAULT_PORT"
    echo "- Rune LFS: http://localhost:$DEFAULT_LFS_PORT"
    echo "- Nginx Proxy: http://localhost:80"
    echo ""
    echo "Management commands:"
    echo "- View logs: docker-compose logs -f"
    echo "- Stop: docker-compose down"
    echo "- Update: docker-compose pull && docker-compose up -d"
}

# Set up SSL certificates
setup_ssl() {
    if [[ -z "$DOMAIN" ]]; then
        error "Domain required for SSL setup. Use -D option."
    fi
    
    log "Setting up SSL for domain: $DOMAIN"
    
    local ssl_dir="$DEFAULT_CONFIG_DIR/ssl"
    mkdir -p "$ssl_dir"
    
    # Check if certbot is available for Let's Encrypt
    if command -v certbot &> /dev/null; then
        log "Using Let's Encrypt for SSL certificate..."
        certbot certonly --standalone -d "$DOMAIN" --non-interactive --agree-tos
        
        # Copy certificates
        cp "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" "$ssl_dir/$DOMAIN.crt"
        cp "/etc/letsencrypt/live/$DOMAIN/privkey.pem" "$ssl_dir/$DOMAIN.key"
        
        # Set up renewal
        echo "0 2 * * * certbot renew --quiet && systemctl restart rune-server" | crontab -
        
    else
        warning "Certbot not found. Generating self-signed certificate..."
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout "$ssl_dir/$DOMAIN.key" \
            -out "$ssl_dir/$DOMAIN.crt" \
            -subj "/C=US/ST=State/L=City/O=Organization/CN=$DOMAIN"
    fi
    
    success "SSL certificates configured for $DOMAIN"
}

# Test server deployment
test_deployment() {
    log "Testing Rune VCS server deployment..."
    
    local api_url="http://localhost:$DEFAULT_PORT"
    local lfs_url="http://localhost:$DEFAULT_LFS_PORT"
    
    # Test API health
    if curl -s -f "$api_url/sync/info" > /dev/null; then
        success "✓ API server responding at $api_url"
    else
        error "✗ API server not responding at $api_url"
    fi
    
    # Test LFS health  
    if curl -s -f "$lfs_url/info" > /dev/null; then
        success "✓ LFS server responding at $lfs_url"
    else
        warning "✗ LFS server not responding at $lfs_url"
    fi
    
    # Test authentication
    if [[ -n "$ADMIN_TOKEN" ]]; then
        if curl -s -f -H "Authorization: Bearer $ADMIN_TOKEN" "$api_url/sync/status" > /dev/null; then
            success "✓ Authentication working"
        else
            warning "✗ Authentication may not be working"
        fi
    fi
    
    success "Server deployment test complete!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            DEFAULT_PORT="$2"
            shift 2
            ;;
        -l|--lfs-port)
            DEFAULT_LFS_PORT="$2"
            shift 2
            ;;
        -d|--data-dir)
            DEFAULT_DATA_DIR="$2"
            shift 2
            ;;
        -c|--config-dir)
            DEFAULT_CONFIG_DIR="$2"
            shift 2
            ;;
        -D|--domain)
            DOMAIN="$2"
            shift 2
            ;;
        -s|--ssl)
            USE_SSL=true
            shift
            ;;
        -t|--token)
            ADMIN_TOKEN="$2"
            shift 2
            ;;
        --no-backup)
            BACKUP_ENABLED=false
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        install|docker|cluster|ssl|backup|test)
            COMMAND="$1"
            shift
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Generate admin token if not provided
if [[ -z "$ADMIN_TOKEN" ]]; then
    ADMIN_TOKEN=$(openssl rand -hex 32)
    warning "Generated admin token: $ADMIN_TOKEN"
    warning "Save this token securely - you'll need it for authentication!"
fi

# Execute command
case "${COMMAND:-install}" in
    install)
        install_standalone
        ;;
    docker)
        deploy_docker
        ;;
    ssl)
        setup_ssl
        ;;
    test)
        test_deployment
        ;;
    *)
        error "Unknown command: $COMMAND"
        ;;
esac
