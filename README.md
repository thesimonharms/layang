# layang

A terminal UI for managing blog posts via a Laravel API.

## Features

- List published posts and drafts in a single view
- Create, edit, publish, unpublish, and delete posts
- Body editing via your preferred external editor
- Excerpt auto-generated from body content, or set manually
- Token-based authentication (held in memory only — never written to disk)

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/layang`.

## Configuration

On first run, layang creates a config file at:

| Platform | Path |
|----------|------|
| Linux/macOS | `~/.config/layang/config.toml` |
| Windows | `%APPDATA%\layang\config.toml` |

```toml
api_url = "https://your-site.com"
editor = "vim"
```

You will be prompted for your API token on every launch. The token is never written to disk.

## Usage

```bash
layang
```

### Keybindings

| Key | Action |
|-----|--------|
| `c` | Create new post |
| `e` | Edit selected post |
| `p` | Publish selected post |
| `u` | Unpublish selected post |
| `d` | Delete selected post |
| `r` | Refresh list |
| `q` / `Ctrl+C` | Quit |

### Post list

Drafts are shown first (in yellow), followed by published posts (in green).

### Create / Edit form

| Key | Action |
|-----|--------|
| `Tab` / `↓` | Next field |
| `Shift+Tab` / `↑` | Previous field |
| `Enter` | Open body in external editor and submit |
| `Esc` | Cancel |

The slug is always auto-generated from the title server-side. The excerpt is auto-generated from the body if left blank.

## API

See [API.MD](API.MD) for the full API specification.
