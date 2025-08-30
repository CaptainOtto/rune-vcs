# 🚀 Rune VCS Server Deployment Guide

Complete guide for deploying Rune VCS anywhere - from home servers to cloud production.

## 🎯 Quick Start (5 minutes)

### Local Development Server

```bash
# Clone and setup
git clone https://github.com/CaptainOtto/rune-vcs.git
cd rune-vcs

# Quick local setup
./quick_start.sh

# Your server is running at:
# - API: http://localhost:8421
# - LFS: http://localhost:8420
# - Web UI: http://localhost:80
```

### Production Server (Any VPS/Cloud)

```bash
# 1. Download setup script
curl -sSL https://raw.githubusercontent.com/CaptainOtto/rune-vcs/main/setup_rune_server.sh -o setup_rune_server.sh
chmod +x setup_rune_server.sh

# 2. Deploy with Docker (recommended)
./setup_rune_server.sh -D your-domain.com -s docker

# 3. Or standalone installation
./setup_rune_server.sh -D your-domain.com -s install
```

## 📋 Deployment Options

### Option 1: Docker (Recommended)

**Best for:** Production, easy scaling, backup management

```bash
# Simple deployment
./setup_rune_server.sh docker

# Production with SSL
./setup_rune_server.sh -D git.mycompany.com -s docker

# Custom ports and configuration
./setup_rune_server.sh -p 80 -l 8080 --token "your-admin-token" docker
```

### Option 2: Standalone Installation

**Best for:** Simple setups, development, learning

```bash
# Local installation
./setup_rune_server.sh install

# Production installation with SSL
sudo ./setup_rune_server.sh -D git.example.com -s install
```

### Option 3: Manual Setup

**Best for:** Custom configurations, existing infrastructure

See [Manual Setup Guide](#manual-setup) below.

## 🌍 Deployment Environments

### Home Server / Self-Hosted

Perfect for personal projects and small teams.

```bash
# Home server with dynamic DNS
./setup_rune_server.sh -D myhomeserver.dyndns.org -p 8080 docker

# Local network only
./setup_rune_server.sh -p 8080 docker
```

**Router Configuration:**

- Port forward 8080 → 8421 (API)
- Port forward 8079 → 8420 (LFS)
- Optional: Port 80 → 80 (Web)

### Cloud VPS (DigitalOcean, Linode, AWS EC2)

Production-ready deployment with SSL.

```bash
# Standard cloud deployment
./setup_rune_server.sh -D git.yourcompany.com -s docker

# High availability setup
./setup_rune_server.sh -D git.yourcompany.com -s cluster
```

**DNS Configuration:**

- A record: `git.yourcompany.com` → `your-server-ip`
- Optional CNAME: `*.git.yourcompany.com` → `git.yourcompany.com`

### Kubernetes/Container Platforms

For enterprise deployments.

```bash
# Generate Kubernetes manifests
./setup_rune_server.sh k8s > rune-deployment.yaml
kubectl apply -f rune-deployment.yaml
```

## 🔧 Configuration

### Environment Variables

Copy and customize `.env.example`:

```bash
cp .env.example .env
# Edit .env with your settings
```

Key settings:

- `RUNE_DOMAIN`: Your server domain
- `RUNE_AUTH_TOKEN`: Admin authentication token
- `POSTGRES_PASSWORD`: Database password
- `SSL_EMAIL`: Email for Let's Encrypt certificates

### Server Configuration

Edit `server.yml` for advanced settings:

```yaml
server:
  host: "0.0.0.0"
  port: 8421

authentication:
  admin_token: "your-secure-token"
  require_auth: true

repositories:
  storage_path: "/data/repositories"
  max_size: "10GB"

backup:
  enabled: true
  interval: "24h"
  s3_bucket: "your-backup-bucket"
```

## 🔐 Security & SSL

### Automatic SSL with Let's Encrypt

```bash
# Enable SSL during setup
./setup_rune_server.sh -D your-domain.com -s docker

# Or add SSL to existing deployment
./setup_rune_server.sh ssl
```

### Manual SSL Certificate

```bash
# Place certificates in ssl/ directory
mkdir ssl
cp your-domain.crt ssl/
cp your-domain.key ssl/

# Update docker-compose.yml
docker-compose restart nginx
```

### Firewall Configuration

```bash
# Ubuntu/Debian
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 8421/tcp  # Rune API (if needed)

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=80/tcp
sudo firewall-cmd --permanent --add-port=443/tcp
sudo firewall-cmd --reload
```

## 👥 User Management

### Add Users (Git-like)

```bash
# Create user account
rune user add alice --email alice@company.com

# Set user permissions
rune user permissions alice --read --write

# Generate access token
rune user token alice
```

### Repository Permissions

```bash
# Create repository
rune repo create myproject

# Add collaborators
rune repo add-user myproject alice --role developer
rune repo add-user myproject bob --role maintainer
```

## 🔄 Using Your Server (Client Side)

### Clone from Your Server

```bash
# Clone repository
rune clone https://git.yourcompany.com/myproject

# With authentication
rune clone https://git.yourcompany.com/myproject --token your-token
```

### Configure Remotes

```bash
# Add remote server
rune remote add origin https://git.yourcompany.com/myproject

# Set authentication
rune remote set-auth origin your-access-token

# Push to server
rune push origin main
```

### Git-like Workflow

```bash
# Your team can use Rune just like Git
rune init
rune add .
rune commit -m "Initial commit"
rune push origin main

# Team members clone and contribute
rune clone https://your-server.com/project
cd project
rune checkout -b feature-branch
# ... make changes ...
rune push origin feature-branch
```

## 📊 Monitoring & Maintenance

### Health Checks

```bash
# Test server health
./setup_rune_server.sh test

# Check all services
curl http://your-server.com/health
curl http://your-server.com/metrics
```

### Logs and Monitoring

```bash
# View logs
docker-compose logs -f rune-server

# Access monitoring dashboard
open http://your-server.com:3000  # Grafana
open http://your-server.com:9090  # Prometheus
```

### Backup & Recovery

```bash
# Manual backup
docker-compose exec rune-server /usr/local/bin/backup.sh

# Restore from backup
docker-compose exec rune-server /usr/local/bin/restore.sh backup-file.tar.gz

# List backups
docker-compose exec rune-server ls -la /backups/
```

## 🔧 Manual Setup

### Prerequisites

- Linux server (Ubuntu 20.04+ recommended)
- Docker and Docker Compose
- Domain name (for SSL)
- 2GB+ RAM, 20GB+ storage

### Step-by-Step Installation

1. **Install Dependencies**

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y docker.io docker-compose curl openssl

# CentOS/RHEL
sudo yum install -y docker docker-compose curl openssl
sudo systemctl enable --now docker
```

2. **Download Rune VCS**

```bash
git clone https://github.com/CaptainOtto/rune-vcs.git
cd rune-vcs
```

3. **Configure Environment**

```bash
cp .env.example .env
# Edit .env with your settings
```

4. **Start Services**

```bash
docker-compose -f docker-compose.production.yml up -d
```

5. **Setup SSL** (Optional)

```bash
./setup_rune_server.sh ssl
```

6. **Test Installation**

```bash
curl http://localhost:8421/sync/info
```

## 🚀 Scaling & High Availability

### Multi-Server Cluster

```bash
# Setup cluster with 3 nodes
./setup_rune_server.sh cluster --nodes 3

# Add load balancer
./setup_rune_server.sh load-balancer --upstream server1,server2,server3
```

### Database Scaling

```bash
# Add read replicas
docker-compose -f docker-compose.cluster.yml up -d

# Enable connection pooling
# Edit server.yml to configure pgbouncer
```

## 🆘 Troubleshooting

### Common Issues

**Server not starting:**

```bash
# Check logs
docker-compose logs rune-server

# Verify ports
netstat -tlnp | grep 8421

# Check permissions
sudo chown -R rune:rune /data
```

**SSL certificate issues:**

```bash
# Renew certificate
certbot renew

# Check certificate
openssl x509 -in ssl/your-domain.crt -text -noout
```

**Authentication problems:**

```bash
# Reset admin token
docker-compose exec rune-server rune admin reset-token

# Test authentication
curl -H "Authorization: Bearer your-token" http://localhost:8421/sync/status
```

### Getting Help

- 📖 Documentation: [docs/](docs/)
- 🐛 Issues: [GitHub Issues](https://github.com/CaptainOtto/rune-vcs/issues)
- 💬 Community: [Discussions](https://github.com/CaptainOtto/rune-vcs/discussions)
- 📧 Support: support@rune-vcs.dev

## 📈 Next Steps

After deployment:

1. **Create your first repository**
2. **Add team members**
3. **Configure backup strategy**
4. **Set up monitoring alerts**
5. **Plan for scaling**

Your Rune VCS server is now ready to replace Git for your team! 🎉

---

## 📋 Quick Reference

### Essential Commands

```bash
# Server management
./setup_rune_server.sh install    # Install server
./setup_rune_server.sh test       # Test deployment
./setup_rune_server.sh ssl        # Setup SSL

# Client usage
rune clone https://server.com/repo     # Clone repository
rune remote add origin https://...     # Add remote
rune push origin main                  # Push changes

# Monitoring
docker-compose logs -f                 # View logs
curl http://server.com/health          # Health check
```

### Important Files

- `docker-compose.production.yml` - Production deployment
- `.env` - Environment configuration
- `server.yml` - Server configuration
- `nginx.conf` - Reverse proxy settings
- `prometheus.yml` - Monitoring configuration

### Default Ports

- 8421: Rune API Server
- 8420: LFS Server
- 80: HTTP (nginx)
- 443: HTTPS (nginx)
- 9090: Prometheus
- 3000: Grafana
