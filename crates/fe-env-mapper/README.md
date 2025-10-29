# FE Environment Mapper

> Dynamic environment variable replacement for frontend applications - Build once, deploy anywhere!

## Overview

**env-mapper** is a CLI tool that solves the problem of hardcoded environment variables in frontend builds. It allows you to use placeholders with default values that get replaced at runtime, enabling true "build once, deploy everywhere" workflow.

### Key Features

- ✅ **Default Value Fallback** - `__VITE_API_URL:http://localhost:3000__`
- ✅ **4 Placeholder Styles** - `__KEY__`, `{{KEY}}`, `${KEY}`, `${{KEY}}`
- ✅ **Include/Exclude Filtering** - Regex or exact match control
- ✅ **Parallel Processing** - Fast multi-file processing
- ✅ **Container-Friendly** - Perfect for Docker/Kubernetes

### How It Works

```typescript
// 1. Build time - Add placeholder with default
const API_URL = "__VITE_API_URL:http://localhost:3000__";

// 2. Runtime - env-mapper replaces based on priority:
//    Environment Variable → Default Value → Keep Placeholder
const API_URL = "https://production-api.com"; // ✅ Replaced
```

---

## Installation

```shell
curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
```

Verify installation:
```shell
env-mapper --help
```

---

## Quick Start

### 1. Add Placeholders to Your Code

```typescript
// src/config.ts
export const config = {
  apiUrl: "__VITE_API_URL:http://localhost:3000__",
  wsUrl: "__VITE_WS_URL:ws://localhost:3001__",
  timeout: __VITE_TIMEOUT:5000__,
  debug: __VITE_DEBUG:false__
};
```

### 2. Create `.env.production` File

```dotenv
VITE_API_URL=__VITE_API_URL__
VITE_WS_URL=__VITE_WS_URL__
VITE_TIMEOUT=__VITE_TIMEOUT__
VITE_DEBUG=__VITE_DEBUG__
```

### 3. Build Your Application

```json
{
  "scripts": {
    "build": "vite build --mode production"
  }
}
```

```shell
npm run build
```

### 4. Run env-mapper

```shell
# Local development (uses defaults)
env-mapper -d ./dist -p .env.production

# Production (uses environment variables)
export VITE_API_URL="https://api.production.com"
export VITE_DEBUG="true"
env-mapper -d ./dist -p .env.production
```

---

## CLI Reference

```shell
env-mapper [OPTIONS]
```

### Core Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `-d, --dir <DIR>` | Directory containing built files | `/js_source_code/dist` | `-d ./dist` |
| `-p, --production-env-file <FILE>` | Template env file with placeholders | `.env.production` | `-p .env.prod` |
| `-e, --runtime-env-file <FILE>` | Runtime env file to override values | None | `-e .env.runtime` |
| `--placeholder <1\|2\|3\|4>` | Placeholder style (1=`__KEY__`, 2=`{{KEY}}`, 3=`${KEY}`, 4=`${{KEY}}`) | `1` | `--placeholder 2` |
| `-s, --suffixes <LIST>` | File extensions to process (comma-separated) | `js,html` | `-s js,html,css` |
| `-o, --output-dir <DIR>` | Output directory (default: overwrite) | None | `-o ./dist-out` |
| `-w, --worker <NUM>` | Number of parallel workers | `1` | `-w 4` |

### Filtering Options

| Option | Description | Example |
|--------|-------------|---------|
| `--include-exact <KEYS>` | Include only specific keys (comma-separated) | `--include-exact "KEY1,KEY2"` |
| `--include <PATTERN>` | Include keys matching regex (comma-separated) | `--include "^VITE_.*,^REACT_APP_.*"` |
| `--exclude-exact <KEYS>` | Exclude specific keys (comma-separated) | `--exclude-exact "SECRET,TOKEN"` |
| `--exclude <PATTERN>` | Exclude keys matching regex (comma-separated) | `--exclude "_SECRET$\|_PASSWORD$"` |

### Advanced Options

| Option | Description | Values |
|--------|-------------|--------|
| `--on-missing <BEHAVIOR>` | Action when no value and no default | `keep-placeholder`, `empty-string`, `error` |
| `-v, --verbose` | Enable verbose logging | Flag |
| `-h, --help` | Show help information | Flag |

---

## Usage Examples

### Example 1: Basic Usage

```shell
env-mapper -d ./dist -p .env.production
```

### Example 2: Docker Deployment

```dockerfile
FROM nginx:alpine
RUN curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
COPY dist/ /usr/share/nginx/html/
COPY .env.production /usr/share/nginx/html/
COPY entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]
```

```bash
#!/bin/sh
# entrypoint.sh
cd /usr/share/nginx/html
env-mapper -d . -p .env.production
nginx -g 'daemon off;'
```

```shell
docker run -e VITE_API_URL=https://api.prod.com myapp:latest
```

### Example 3: With Runtime Override File

```shell
# .env.runtime
VITE_API_URL=https://api.staging.com
VITE_DEBUG=true

env-mapper -d ./dist -p .env.production -e .env.runtime
```

### Example 4: Security Filtering

```shell
# Exclude sensitive variables
env-mapper -d ./dist -p .env.production \
  --exclude "_SECRET$|_PASSWORD$|_TOKEN$|_KEY$"
```

### Example 5: Multi-Framework Support

```shell
# Support multiple frameworks
env-mapper -d ./dist -p .env.production \
  --include "^VITE_|^REACT_APP_|^NEXT_PUBLIC_"
```

### Example 6: Parallel Processing

```shell
# Process with 8 workers for faster execution
env-mapper -d ./dist -p .env.production -w 8
```

### Example 7: Different Placeholder Styles

```shell
# Use double curly braces style {{KEY}}
env-mapper -d ./dist -p .env.production --placeholder 2

# Use dollar curly style ${KEY}
env-mapper -d ./dist -p .env.production --placeholder 3
```

---

## Placeholder Styles

```javascript
// Style 1: Underscore (default)
const url = "__VITE_API_URL:http://localhost:3000__";

// Style 2: Double Curly
const url = "{{VITE_API_URL:http://localhost:3000}}";

// Style 3: Dollar Curly
const url = "${VITE_API_URL:http://localhost:3000}";

// Style 4: Dollar Brace
const url = "${{VITE_API_URL:http://localhost:3000}}";
```

---

## Replacement Priority

```
1. Environment Variable (highest priority)
   ↓ (if not found or empty)
2. Default Value from Placeholder
   ↓ (if no default)
3. Keep Original Placeholder (or based on --on-missing flag)
```

---

## Complete Documentation

📖 **[Full Guide](./GUIDE.md)** - Comprehensive documentation with advanced examples, best practices, and troubleshooting

---

## Examples Repository

Full working examples: [examples/env-mapper](https://github.com/TikTzuki/tiktuzki-scripts/tree/master/examples/env-mapper)

---

## License

Apache License 2.0
