# megui

A simple artworks viewer and portfolio built with Rust, egui, and eframe.

## Features

- **Artworks Gallery**: Browse and view artworks from your collection
- **Resume Viewer**: Display your resume with markdown rendering
- **Theme Support**: Auto, light, and dark modes
- **Deep Linking**: Direct URLs to specific pages
- **Multiple Artwork Windows**: Open and view multiple artworks simultaneously
- **Cross-Platform**: Runs natively and on the web (WASM)

## Project Structure

```
megui/
├── src/
│   ├── lib.rs           # Application entry point
│   ├── app.rs           # Main application logic, views, and UI
│   ├── config.rs        # Configuration loader
│   ├── artwork.rs       # Artwork data structures
├── config.toml          # Application configuration
├── index.html           # Web entry point
├── Cargo.toml           # Rust dependencies
└── .github/
    └── workflows/
        └── pages.yml    # GitHub Actions deployment
```

## Configuration

All application settings are in `config.toml`:

```toml
[app]
name = "hwww"
website = "https://hwww.org"
resume = "https://resume.hwww.org"
artworks = "https://artworks.hwww.org/index.json"
repository = "https://github.com/4www/megui"
default_theme = "auto"  # Options: "auto", "dark", "light"
```

## Development

### Prerequisites

- Rust (latest stable)
- wasm-pack (for web builds): `cargo install wasm-pack`

### Running Locally

**Native:**
```bash
cargo run
```

**Web (development):**
```bash
# You need a local web server to serve the `dist` directory
# For example, using `python -m http.server` in the `dist` directory
wasm-pack build --target web --out-name wasm --out-dir ./dist
```

**Web (production build):**
```bash
wasm-pack build --target web --out-name wasm --out-dir ./dist --release
```

### Building

**Native binary:**
```bash
cargo build --release
```

**Web (WASM):**
```bash
wasm-pack build --target web --out-name wasm --out-dir ./dist --release
```

## Deployment

The project automatically builds and deploys to GitHub Pages on push to the `main` branch using GitHub Actions.

1. Push changes to the `main` branch.
2. The GitHub Actions workflow (`.github/workflows/pages.yml`) will build the project and deploy it to GitHub Pages.
3. The site will be available at: `https://<username>.github.io/<repo-name>/`

## Tech Stack

- **[Rust](https://www.rust-lang.org/)** - Programming language
- **[egui](https://www.egui.rs/)** - Immediate mode GUI framework
- **[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** - Web and native support for egui
- **[hframe](https://github.com/lucasmerlin/hello_egui/tree/main/crates/hframe)** - HTML iframe support for egui
- **[wasm-pack](https://rustwasm.github.io/wasm-pack/)** - A tool for building and working with Rust-generated WebAssembly

## Routes

The application supports the following routes (on web):

- `#/artworks` - Artworks gallery view
- `#/resume` - Resume viewer
- `#/about` - About page

## License

See repository for license information.
