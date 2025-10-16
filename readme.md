# 🛠️ Anvil - Universal Template Engine

[![Crates.io](https://img.shields.io/crates/v/anvil-cli.svg)](https://crates.io/crates/anvil-cli)
[![Documentation](https://docs.rs/anvil-engine/badge.svg)](https://docs.rs/anvil-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful, universal template engine for developers. Generate production-ready projects with intelligent service composition and cross-platform support.

## 🚀 Quick Start

### Installation

```bash
cargo install anvil-cli
```

### Usage

```bash
# Create a new full-stack SaaS project
anvil create my-saas --template fullstack-saas

# Create with specific services
anvil create my-app --template fullstack-saas --auth clerk --api rest

# Use a preset configuration
anvil create my-app --template fullstack-saas --preset "Starter Pack"

# List available templates
anvil list
```

## ✨ Features

- 🎯 **Service Composition**: Mix and match authentication, payments, databases, AI, and more
- 🔧 **Multi-Language**: Templates for TypeScript, Rust, Go, and more
- 📦 **Preset Configurations**: Quick-start with pre-configured service combinations
- 🚀 **Production Ready**: Generate fully functional, deployable applications
- 🧪 **Tested**: Comprehensive integration tests across all platforms
- 🌐 **Cross-Platform**: Works on Linux, macOS, and Windows

## 📚 Documentation

- **[Comprehensive Documentation](https://docs.useanvil.tech)**

## 🛠️ Available Templates

- **fullstack-saas**: Next.js 14 + TypeScript SaaS template
- **rust-web-api**: Axum web API template
- **rust-hello-world**: Simple Rust CLI
- **go-cli-tool**: Go CLI with Cobra framework

## 🔌 Service Integrations

### Authentication
- Clerk, Auth0, Firebase, Supabase

### Payments
- Stripe

### Databases
- Neon, PlanetScale, MongoDB, Supabase

### AI
- OpenAI, Claude, Replicate

### API Patterns
- REST (with OpenAPI), tRPC, GraphQL

### Deployment
- Vercel, Railway, Render, Docker

## 🧩 Example

```bash
# Create a SaaS with authentication, payments, and AI
anvil create my-startup \
  --template fullstack-saas \
  --auth clerk \
  --payments stripe \
  --ai openai \
  --database neon \
  --deployment vercel
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a PR.

## 📝 License

This project is licensed under the MIT license ([LICENSE](LICENSE))
