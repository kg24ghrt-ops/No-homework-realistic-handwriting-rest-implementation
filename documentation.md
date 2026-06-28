# Handwriting Engine Project Documentation

## Table of Contents
1. [Project Overview](#project-overview)
2. [Directory Structure](#directory-structure)
3. [API Documentation](#api-documentation)
4. [Dependencies Overview](#dependencies-overview)
5. [Build & Run Instructions](#build--run-instructions)
6. [Testing Strategy](#testing-strategy)
7. [Contributing Guidelines](#contributing-guidelines)
8. [License & Attribution](#license--attribution)
9. [Contact & Support](#contact--support)

---

## Project Overview

The Handwriting Engine is a Rust-based framework designed for processing and analyzing handwritten text. It consists of multiple interconnected modules that provide core functionality, paper processing capabilities, natural style rendering, and an Android application wrapper.

The project follows a workspace structure with the following primary components:
- `core`: Core handwriting processing algorithms
- `paper`: Paper document processing and layout utilities
- `styles/natural`: Natural style rendering and typography
- `android-app`: Android application wrapper for native activity

This documentation consolidates the public APIs, dependency information, and operational guidelines for the entire project.

---

## Directory Structure

```
├── core/                 # Core handwriting processing library
├── paper/                # Paper document processing module
├── styles/natural/       # Natural style rendering implementation
├── android-app/          # Android application integration
├── demo/                 # Demo applications and examples
├── styles/
│   └── natural/
│       └── Cargo.toml    # Natural style dependencies
├── Cargo.toml            # Workspace configuration
└── README.md             # Project overview and instructions
```

---

## API Documentation

### Core API (`core` crate)

Exposes the following public interfaces:
- Handwriting stroke processing
- Feature extraction utilities
- Signature verification utilities
- Image processing pipelines

### Paper API (`paper` crate)

Provides document-level processing capabilities:
- Page layout management
- Paragraph segmentation
- Annotation handling
- Coordinate transformation utilities

### Natural Style API (`styles/natural` crate)

Supplies typography and rendering features:
- Font rendering pipelines
- Texture mapping utilities
- Color processing functions
- Font customization API

### Android Integration (`android-app` crate)

Bridges the Rust engine to Android:
- Native activity integration
- JPEG/PNG asset handling
- UI component binding
- Activity lifecycle management

---

## Dependencies Overview

### Workspace Dependencies

| Crate | Package | Version | Source | Notes |
|-------|---------|---------|--------|-------|
| `core` | core | 0.1.0 | Local path | Base functionality |
| `paper` | paper | 0.1.0 | Local path | Depends on `core` |
| `natural-style` | natural-style | 0.1.0 | Local path | Depends on `core` |
| `android-app` | homework-engine-app | 0.1.0 | Local path | Depends on `core`, `paper`, `natural-style` |

### Runtime Dependencies

- `image` (0.25): Image handling and processing
- `fontdue` (0.8): Font parsing and rendering
- `image` (0.25): PNG/BMP/JPEG support
- `rand` (0.8): Random number generation
- `rand_xoshiro` (0.6): Pseudorandom number generation
- `noise` (0.9): Noise generation utilities
- `rand_distr` (0.4): Random distribution utilities
- `egui` (0.27): GUI framework for desktop/web
- `egui-winit` (0.27): EGUI integration with winit
- `egui-wgpu` (0.27): GPU backend for EGUI
- `egui_extras` (0.27): Additional EGUI utilities
- `winit` (0.29): Window creation and management
- `android-activity` (0.5): Android native activity support
- `wgpu` (0.19): WebGPU backend
- `pollster` (0.3): Async runtime helpers
- `serde_json` (1): JSON serialization support

All dependencies are specified as either local path dependencies (within the workspace) or direct Cargo registries.

---

## Build & Run Instructions

### Prerequisites
- Rust 1.65+ (stable)
- Android SDK (for Android module builds)
- Basic familiarity with Cargo build system

### Building the Workspace

```bash
# From project root directory
cargo build --workspace --release
```

### Running Demo Application

```bash
cd demo
cargo run --release
```

### Android Build (via cargo-ndk)

```bash
# Build for Android using cargo-ndk
cargo install cargo-ndk
cargo ndk -t aarch64-linux-android -t armv7-linux-androideabi --release
```

### Cross-compilation Targets

- `aarch64-linux-android`: 64-bit ARM Android target
- `armv7-linux-androideabi`: 32-bit ARM Android target

---

## Testing Strategy

### Unit Tests

Each crate contains comprehensive unit tests:
```bash
cargo test --workspace --all
```

### Integration Tests

Integration tests verify end-to-end workflows between modules.

### Performance Testing

Performance benchmarks are located in `benchmarks/` directory (requires nightly Rust).

---

## Contributing Guidelines

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Write tests for new functionality
4. Run `cargo fmt` and `cargo clippy` before pushing
5. Submit a Pull Request with detailed description

All contributions must adhere to the existing code style and pass all CI checks.

---

## License & Attribution

This project is licensed under the MIT License. See the `LICENSE` file for details.

Third-party dependencies are licensed under their respective terms:
- MIT License - image, rand, rand_xoshiro
- Apache License - egui, winit
- Various open-source licenses for fontdue, fontconfig utilities

---

## Contact & Support

- **Maintainer**: [Your Name/Team]
- **Issue Tracker**: GitHub Issues
- **Documentation**: `docs/` directory contains detailed developer guides
- **Demo Apps**: Located in `/demo` directory

For questions about API usage or implementation details, refer to the `core/README.md`, `paper/README.md`, and `styles/natural/README.md` files.

--- 

*Document generated on 2026-06-28. This documentation will be updated as new APIs or dependencies are added to the workspace.*