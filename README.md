# 2Wee Web Terminal

A self-contained WebSocket bridge that runs a [2Wee](https://2wee.dev) TUI session in any browser. It spawns `two_wee_client` over a PTY, streams it via WebSocket, and serves the xterm.js frontend — all from a single binary.

## Quick start

Download the binary for your platform from [Releases](https://github.com/2wee-dev/web-terminal/releases/latest):

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `two_wee_terminal-macos-arm64` |
| macOS (Intel) | `two_wee_terminal-macos-x86_64` |
| Linux x86_64 | `two_wee_terminal-linux-x86_64` |
| Linux ARM64 | `two_wee_terminal-linux-arm64` |

```bash
chmod +x two_wee_terminal-macos-arm64
TWO_WEE_SERVER=https://your-app.example.com ./two_wee_terminal-macos-arm64
```

Open `http://localhost:7681` in your browser.

## Configuration

| Environment variable | Default | Description |
|----------------------|---------|-------------|
| `TWO_WEE_SERVER` | — | Lock the terminal to a server URL. If not set, a landing page lets users enter the URL. |
| `TWO_WEE_PORT` | `7681` | Port to listen on |
| `TWO_WEE_CLIENT_BIN` | `two_wee_client` | Path to the `two_wee_client` binary |

## Laravel integration

If you use Laravel, the [2wee/laravel](https://github.com/2wee-dev/laravel) package manages the terminal for you:

```bash
php artisan 2wee:install-terminal
php artisan 2wee:start-terminal
```

Then add the Blade component to any view:

```blade
<x-2wee::terminal />
```

## Production (Nginx)

Proxy the WebSocket and terminal script through your existing HTTPS server:

```nginx
location /ws {
    proxy_pass         http://127.0.0.1:7681;
    proxy_http_version 1.1;
    proxy_set_header   Upgrade $http_upgrade;
    proxy_set_header   Connection "upgrade";
    proxy_read_timeout 3600s;
}

location /terminal.js {
    proxy_pass http://127.0.0.1:7681;
}
```

## Build from source

```bash
git clone https://github.com/2wee-dev/web-terminal.git
cd web-terminal
npm install --prefix frontend
cargo build --release
```

## Documentation

Full documentation at [2wee.dev](https://2wee.dev/introduction).
