# 🏠 Rune VCS Home Server Setup Guide

Set up your own Rune VCS server anywhere - at home, on a VPS, or in the cloud. Works just like Git but with Rune's advanced features.

## 🚀 Quick Start (1 minute setup)

```bash
# Clone Rune VCS
git clone https://github.com/CaptainOtto/rune-vcs.git
cd rune-vcs

# Run automated setup
./deploy_server.sh

# That's it! Your server is running on http://localhost:8080
```

## 📋 Prerequisites

- **Docker** and **Docker Compose** installed
- **Port 8080** available (or customize)
- **2GB+ RAM** recommended
- **10GB+ disk space** for repositories

## 🔧 Custom Setup

### 1. Basic Home Server

```bash
# Setup on custom port
./deploy_server.sh my-home-server 9000

# Setup with domain
./deploy_server.sh production 80 your-domain.com
```

### 2. Manual Configuration

```bash
# Create server directory
mkdir -p ~/rune-servers/my-server
cd ~/rune-servers/my-server

# Copy configuration template
cp /path/to/rune-vcs/.env.template .env

# Edit configuration
nano .env

# Start with Docker
docker-compose -f /path/to/rune-vcs/docker-compose.multi-server.yml up -d
```

### 3. Production Setup

```bash
# Enable SSL and production settings
RUNE_SSL_ENABLED=true ./deploy_server.sh production 443 your-domain.com
```

## 🖥️ Client Setup

### Connect to Your Server

```bash
# Add your server as a remote
rune remote add origin http://your-server:8080 --token YOUR_AUTH_TOKEN

# Clone repositories
rune clone http://your-server:8080/my-repo ./my-repo

# Work normally
cd my-repo
rune add .
rune commit -m "First commit"
rune push origin main
```

### Multiple Servers

```bash
# Add multiple remotes
rune remote add home http://home-server:8080 --token HOME_TOKEN
rune remote add cloud http://cloud-server:443 --token CLOUD_TOKEN

# Sync between servers
rune push home main
rune push cloud main
```

## 🌐 Network Configuration

### Home Network Setup

1. **Router Port Forwarding**: Forward port 8080 to your server
2. **Dynamic DNS**: Use services like DuckDNS for dynamic IP
3. **Firewall**: Allow port 8080 (or your custom port)

```bash
# Example with custom domain
./deploy_server.sh home-server 8080 myserver.duckdns.org
```

### VPS/Cloud Setup

```bash
# Amazon EC2, DigitalOcean, etc.
./deploy_server.sh cloud-server 80 your-vps-ip

# With SSL certificate
RUNE_SSL_ENABLED=true ./deploy_server.sh secure-server 443 your-domain.com
```

## 🔒 Security Best Practices

### 1. Authentication

```bash
# Generate strong auth token
openssl rand -hex 32

# Set in .env file
RUNE_AUTH_TOKEN=your-generated-token
```

### 2. SSL/TLS Setup

```bash
# Get SSL certificate (Let's Encrypt)
certbot certonly --standalone -d your-domain.com

# Configure in .env
RUNE_SSL_ENABLED=true
RUNE_SSL_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
RUNE_SSL_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem
```

### 3. Firewall Configuration

```bash
# Ubuntu/Debian
sudo ufw allow 8080
sudo ufw enable

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload
```

## 📊 Monitoring & Maintenance

### Health Checks

```bash
# Check server status
curl http://your-server:8080/health

# View logs
docker-compose logs -f

# Monitor resources
docker stats
```

### Backup & Restore

```bash
# Backup repositories
tar -czf backup-$(date +%Y%m%d).tar.gz data/repositories/

# Restore from backup
tar -xzf backup-20240828.tar.gz
```

### Updates

```bash
# Update Rune server
cd ~/rune-servers/my-server
docker-compose pull
docker-compose up -d
```

## 🔧 Troubleshooting

### Common Issues

**Server won't start:**

```bash
# Check logs
docker-compose logs rune-server

# Check port availability
netstat -tlnp | grep 8080
```

**Can't connect from client:**

```bash
# Test connectivity
curl http://your-server:8080/health

# Check firewall
sudo ufw status
```

**Performance issues:**

```bash
# Monitor resources
docker stats

# Increase memory limits in docker-compose.yml
deploy:
  resources:
    limits:
      memory: 4G
```

### Log Analysis

```bash
# Follow logs in real-time
docker-compose logs -f rune-server

# Search for errors
docker-compose logs rune-server | grep ERROR

# Export logs
docker-compose logs rune-server > server.log
```

## 🌟 Advanced Features

### Multi-Server Setup

```bash
# Setup load-balanced cluster
./deploy_server.sh server1 8081
./deploy_server.sh server2 8082

# Configure nginx load balancer
# (automatically included in multi-server compose)
```

### Repository Management

```bash
# Create repository on server
curl -X POST http://your-server:8080/repositories \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-project"}'

# List repositories
curl http://your-server:8080/repositories \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### User Management (Multi-User)

```bash
# Enable PostgreSQL in docker-compose.yml
# Create users via API
curl -X POST http://your-server:8080/users \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "email": "alice@example.com"}'
```

## 📚 Integration Examples

### Git Migration

```bash
# Convert Git repository to Rune
git clone https://github.com/user/repo.git
cd repo
rune init
rune add .
rune commit -m "Migrated from Git"
rune remote add origin http://your-server:8080/repo
rune push origin main
```

### CI/CD Integration

```yaml
# GitHub Actions example
name: Deploy to Rune
on: [push]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Deploy to Rune
        run: |
          rune remote add production http://your-server:8080/app --token ${{ secrets.RUNE_TOKEN }}
          rune push production main
```

### Backup Automation

```bash
#!/bin/bash
# backup.sh - Run daily via cron
cd ~/rune-servers/my-server
tar -czf /backups/rune-$(date +%Y%m%d).tar.gz data/
# Upload to cloud storage
aws s3 cp /backups/rune-$(date +%Y%m%d).tar.gz s3://my-backups/
```

## 🆘 Support

- **Documentation**: [docs.rune-vcs.dev](https://docs.rune-vcs.dev)
- **Issues**: [GitHub Issues](https://github.com/CaptainOtto/rune-vcs/issues)
- **Community**: [Discord Server](https://discord.gg/rune-vcs)
- **Email**: support@rune-vcs.dev

## 📄 License

Rune VCS is open source under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

_Ready to revolutionize your version control? Start your Rune server today!_ 🚀
