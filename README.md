# megui

A simple "Hello World" application built with [egui](https://github.com/emilk/egui/).

## Development

### Prerequisites
- Rust toolchain

### Running locally

Native:
```bash
cargo run
```

Web (with hot reload):
```bash
# Install Trunk if needed (globally on the system)
cargo install --locked trunk

# Run local dev server
trunk serve
```

Then open http://127.0.0.1:8080/index.html

### Building for production

Native:
```bash
cargo build --release
```

Web:
```bash
trunk build --release
```

The web build will be in the `dist/` directory.

## Deployment

This project is configured to automatically deploy to GitHub Pages when pushed to the `main` branch.

### Setup GitHub Pages

1. Go to your repository settings
2. Navigate to Pages
3. Under "Build and deployment", select "GitHub Actions" as the source
4. Push to main branch and the workflow will automatically build and deploy

The site will be available at: `https://<username>.github.io/megui/`
