# 📚 Environment Mapper - Complete Guide

## 🎯 Overview

**env-mapper** is a powerful CLI tool designed to solve the dynamic environment variable problem in frontend applications. It allows you to build your frontend application once and deploy it to multiple environments (dev, staging, production) without rebuilding.

### The Problem It Solves

When you build a frontend application (React, Vue, Angular), environment variables are typically "baked in" at build time:

```typescript
// At build time, this becomes a hardcoded string
const API_URL = import.meta.env.VITE_API_URL; // → "https://dev-api.com"
```

This means you need to rebuild for each environment, which is:
- ❌ Time-consuming
- ❌ Error-prone
- ❌ Not container-friendly
- ❌ Violates the "build once, deploy anywhere" principle

### The Solution

env-mapper uses **placeholder patterns with default values** that can be replaced at runtime:

```typescript
// Build time - use placeholder with default
const API_URL = "__VITE_API_URL:http://localhost:3000__";

// After env-mapper runs (runtime)
const API_URL = "https://production-api.com"; // ✅ Replaced with actual env value
```

---

## 🚀 Installation

```bash
curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
```

Verify installation:
```bash
env-mapper --help
```

---

## 📖 Core Concepts

### 1. Placeholder Styles

env-mapper supports 4 different placeholder styles to match your preference or framework conventions:

| Style | Syntax | Example | Common Usage |
|-------|--------|---------|--------------|
| **Underscore** (default) | `__KEY__` or `__KEY:default__` | `__VITE_API_URL:http://localhost:3000__` | Vite, general purpose |
| **Double Curly** | `{{KEY}}` or `{{KEY:default}}` | `{{VITE_API_URL:http://localhost:3000}}` | Handlebars, Mustache |
| **Dollar Curly** | `${KEY}` or `${KEY:default}` | `${VITE_API_URL:http://localhost:3000}` | Shell-like, template strings |
| **Dollar Brace** | `${{KEY}}` or `${{KEY:default}}` | `${{VITE_API_URL:http://localhost:3000}}` | GitHub Actions style |

### 2. Default Values

Default values are specified **inside** the placeholder using colon (`:`) separator:

```javascript
// Syntax: PLACEHOLDER_START + KEY:DEFAULT_VALUE + PLACEHOLDER_END
__VITE_API_URL:http://localhost:3000__
```

**How it works:**
1. env-mapper looks for environment variable `VITE_API_URL`
2. If found and not empty → use env value
3. If not found → use default value `http://localhost:3000`
4. If no env value and no default → keep placeholder as-is

### 3. Replacement Priority

```
1. Environment Variable (highest priority)
   ↓ (if not found or empty)
2. Default Value from Placeholder
   ↓ (if no default)
3. Keep Original Placeholder (or empty/error based on --on-missing flag)
```

---

## 🎨 Usage Examples

### Basic Example

**Step 1: Prepare your code with placeholders**

```typescript
// src/config.ts
export const config = {
  apiUrl: "__VITE_API_URL:http://localhost:3000__",
  wsUrl: "__VITE_WS_URL:ws://localhost:3001__",
  timeout: __VITE_TIMEOUT:5000__,
  debug: __VITE_DEBUG:false__,
};
```

**Step 2: Create `.env.production` file**

```env
VITE_API_URL=__VITE_API_URL__
VITE_WS_URL=__VITE_WS_URL__
VITE_TIMEOUT=__VITE_TIMEOUT__
VITE_DEBUG=__VITE_DEBUG__
```

**Step 3: Build your application**

```bash
npm run build
# Creates dist/ folder with placeholders intact
```

**Step 4: Run env-mapper**

```bash
# Without environment variables (uses defaults)
env-mapper -d ./dist -p .env.production

# With environment variables (overrides defaults)
export VITE_API_URL="https://api.production.com"
export VITE_DEBUG="true"
env-mapper -d ./dist -p .env.production
```

**Result:**
```typescript
// dist/assets/index-abc123.js
export const config = {
  apiUrl: "https://api.production.com",  // ✅ From env
  wsUrl: "ws://localhost:3001",          // ✅ From default
  timeout: 5000,                         // ✅ From default
  debug: true,                           // ✅ From env
};
```

---

## 🔧 CLI Reference

### Basic Command

```bash
env-mapper [OPTIONS]
```

### Core Options

#### `-d, --dir <DIR>`
Directory containing built files to process.

```bash
env-mapper -d ./dist
env-mapper -d /app/public
```

**Default:** `/js_source_code/dist`

---

#### `-p, --production-env-file <FILE>`
File containing placeholder patterns (template).

```bash
env-mapper -p .env.production
env-mapper -p /config/.env.template
```

**Default:** `.env.production`

**File format:**
```env
VITE_API_URL=__VITE_API_URL__
VITE_APP_NAME=__VITE_APP_NAME__
```

---

#### `--placeholder <1|2|3|4>`
Placeholder style to use.

```bash
env-mapper --placeholder 1  # __KEY__ (default)
env-mapper --placeholder 2  # {{KEY}}
env-mapper --placeholder 3  # ${KEY}
env-mapper --placeholder 4  # ${{KEY}}
```

**Default:** `1` (Underscore style)

---

#### `-e, --runtime-env-file <FILE>`
Runtime environment file to override system environment variables.

```bash
env-mapper -e .env.runtime
env-mapper -e /secrets/.env.prod
```

**Use case:** Override specific values without changing system env vars.

**Example `.env.runtime`:**
```env
VITE_API_URL=https://api.staging.com
VITE_DEBUG=true
```

---

#### `-s, --suffixes <SUFFIXES>`
File extensions to process (comma-separated).

```bash
env-mapper -s js,html
env-mapper -s js,html,css,json
```

**Default:** `js,html`

---

#### `-o, --output-dir <DIR>`
Output directory for processed files (preserves original if not specified).

```bash
# Overwrite original files
env-mapper -d ./dist

# Write to new directory
env-mapper -d ./dist -o ./dist-processed
```

**Default:** Overwrites original files

---

#### `-w, --worker <NUMBER>`
Number of parallel workers for processing files.

```bash
env-mapper -w 1   # Sequential processing
env-mapper -w 4   # Process 4 files in parallel
env-mapper -w 8   # Process 8 files in parallel
```

**Default:** `1`

**Recommendation:** Use CPU count or less for optimal performance.

---

### Filtering Options

Control which environment variables get replaced using include/exclude filters.

#### `--include-exact <KEYS>`
Include only specific keys (exact match, comma-separated).

```bash
env-mapper --include-exact "VITE_API_URL,VITE_APP_NAME,VITE_VERSION"
```

---

#### `--include <PATTERNS>`
Include keys matching regex patterns (comma-separated).

```bash
# Include all VITE_ variables
env-mapper --include "^VITE_.*"

# Include multiple patterns
env-mapper --include "^VITE_.*,^REACT_APP_.*,^NEXT_PUBLIC_.*"
```

---

#### `--exclude-exact <KEYS>`
Exclude specific keys (exact match, comma-separated).

```bash
env-mapper --exclude-exact "VITE_SECRET_KEY,VITE_INTERNAL_TOKEN"
```

---

#### `--exclude <PATTERNS>`
Exclude keys matching regex patterns (comma-separated).

```bash
# Exclude all secret/sensitive variables
env-mapper --exclude "_SECRET$|_PASSWORD$|_TOKEN$|_KEY$"

# Exclude internal and debug variables
env-mapper --exclude "_INTERNAL_|_DEBUG_"
```

---

### Advanced Options

#### `--on-missing <keep-placeholder|empty-string|error>`
Behavior when no env value and no default value exists.

```bash
# Keep placeholder as-is (default, safest)
env-mapper --on-missing keep-placeholder

# Replace with empty string
env-mapper --on-missing empty-string

# Fail with error (strict mode)
env-mapper --on-missing error
```

**Default:** `keep-placeholder`

---

#### `-v, --verbose`
Enable verbose logging.

```bash
env-mapper -v -d ./dist -p .env.production
```

---

## 📋 Real-World Scenarios

### Scenario 1: Local Development

```bash
# Use all default values from placeholders
env-mapper -d ./dist -p .env.production
```

**Result:** All placeholders use their default values (typically localhost URLs).

---

### Scenario 2: Docker Container Deployment

**Dockerfile:**
```dockerfile
FROM node:18 AS builder
WORKDIR /app
COPY . .
RUN npm ci && npm run build

FROM nginx:alpine
# Install env-mapper
RUN apk add --no-cache curl bash && \
    curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash

COPY --from=builder /app/dist /usr/share/nginx/html
COPY .env.production /usr/share/nginx/html/

# Entrypoint script
COPY docker-entrypoint.sh /
RUN chmod +x /docker-entrypoint.sh
ENTRYPOINT ["/docker-entrypoint.sh"]
```

**docker-entrypoint.sh:**
```bash
#!/bin/sh
cd /usr/share/nginx/html
env-mapper -d . -p .env.production
nginx -g 'daemon off;'
```

**Run container:**
```bash
docker run -e VITE_API_URL=https://api.prod.com \
           -e VITE_DEBUG=false \
           myapp:latest
```

---

### Scenario 3: Kubernetes Deployment

**ConfigMap:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  VITE_API_URL: "https://api.k8s.example.com"
  VITE_WS_URL: "wss://ws.k8s.example.com"
  VITE_TIMEOUT: "10000"
```

**Deployment:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend-app
spec:
  template:
    spec:
      initContainers:
      - name: env-mapper
        image: myapp:latest
        command: ["/bin/sh", "-c"]
        args:
          - |
            cd /app/dist
            env-mapper -d . -p .env.production
        envFrom:
        - configMapRef:
            name: app-config
        volumeMounts:
        - name: app-dist
          mountPath: /app/dist
      containers:
      - name: nginx
        image: nginx:alpine
        volumeMounts:
        - name: app-dist
          mountPath: /usr/share/nginx/html
      volumes:
      - name: app-dist
        emptyDir: {}
```

---

### Scenario 4: Multi-Environment with Filtering

**Use case:** You have different variables for different environments and want to control which ones get replaced.

```bash
# Production: Only replace production-safe variables
env-mapper \
  -d ./dist \
  -p .env.production \
  --include "^VITE_(API_URL|APP_NAME|VERSION)$" \
  --exclude "_DEBUG|_INTERNAL"

# Development: Replace all including debug variables
env-mapper \
  -d ./dist \
  -p .env.production \
  --include "^VITE_.*"
```

---

### Scenario 5: Security - Exclude Sensitive Variables

```bash
# Ensure sensitive variables are never replaced in dist files
env-mapper \
  -d ./dist \
  -p .env.production \
  --exclude "_SECRET$|_PASSWORD$|_TOKEN$|_PRIVATE_KEY$|_CREDENTIAL"
```

---

## 🎯 Best Practices

### 1. Always Use Default Values

```typescript
// ✅ Good - has fallback for local development
const API_URL = "__VITE_API_URL:http://localhost:3000__";

// ❌ Bad - breaks if env var not set
const API_URL = "__VITE_API_URL__";
```

### 2. Use Descriptive Default Values

```typescript
// ✅ Good - clear what it's for
const TIMEOUT = __VITE_REQUEST_TIMEOUT:5000__; // milliseconds

// ❌ Bad - unclear units or purpose
const TIMEOUT = __VITE_TIMEOUT:5__; // 5 what?
```

### 3. Keep Secrets Out of Defaults

```typescript
// ❌ NEVER put real secrets in defaults
const API_KEY = "__VITE_API_KEY:sk_live_abc123__"; // ❌ DANGEROUS!

// ✅ Good - use placeholder or empty default
const API_KEY = "__VITE_API_KEY__"; // Must be provided via env
```

### 4. Use Filtering for Security

```bash
# Always exclude sensitive patterns
env-mapper \
  --exclude "_SECRET|_PASSWORD|_TOKEN|_KEY$|_CREDENTIAL"
```

### 5. Document Your Placeholders

Create a `ENV_VARIABLES.md` in your project:

```markdown
# Environment Variables

| Variable | Default | Description | Required |
|----------|---------|-------------|----------|
| VITE_API_URL | http://localhost:3000 | Backend API URL | No |
| VITE_WS_URL | ws://localhost:3001 | WebSocket URL | No |
| VITE_TIMEOUT | 5000 | Request timeout (ms) | No |
| VITE_SECRET_KEY | - | API secret key | Yes |
```

### 6. Test Your Placeholders

```bash
# Test 1: Without env vars (should use defaults)
env-mapper -d ./dist -p .env.production -o ./dist-test1

# Test 2: With env vars (should use env values)
export VITE_API_URL="https://test.com"
env-mapper -d ./dist -p .env.production -o ./dist-test2

# Test 3: Strict mode (should fail if required vars missing)
env-mapper -d ./dist -p .env.production --on-missing error
```

### 7. Use in CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build
        run: npm ci && npm run build
      
      - name: Install env-mapper
        run: curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
      
      - name: Replace env variables
        env:
          VITE_API_URL: ${{ secrets.PRODUCTION_API_URL }}
          VITE_APP_NAME: ${{ vars.APP_NAME }}
        run: |
          env-mapper -d ./dist -p .env.production
      
      - name: Deploy
        run: # your deployment command
```

---

## 🔍 Troubleshooting

### Issue: Placeholders not being replaced

**Symptoms:**
```javascript
// After env-mapper, still see:
const url = "__VITE_API_URL:http://localhost:3000__";
```

**Solutions:**

1. **Check placeholder style matches:**
```bash
# If using {{KEY}}, specify style 2
env-mapper --placeholder 2 -d ./dist -p .env.production
```

2. **Check file extensions:**
```bash
# Make sure file extension is included
env-mapper -s js,html,css -d ./dist -p .env.production
```

3. **Check .env.production file:**
```env
# Must have the key listed
VITE_API_URL=__VITE_API_URL__  # ✅ Correct
```

4. **Check filtering rules:**
```bash
# Key might be excluded
env-mapper -d ./dist -p .env.production --verbose
```

---

### Issue: Wrong value being used

**Symptoms:**
```javascript
// Expected production URL but got default
const url = "http://localhost:3000"; // Should be https://api.prod.com
```

**Solutions:**

1. **Check environment variable is set:**
```bash
echo $VITE_API_URL  # Should print the value
```

2. **Check env value is not empty:**
```bash
export VITE_API_URL=""  # ❌ Empty string = use default
export VITE_API_URL="https://api.prod.com"  # ✅ Correct
```

3. **Check runtime env file:**
```bash
# If using -e flag, check that file
cat .env.runtime
```

---

### Issue: Build breaks after adding placeholders

**Symptoms:**
```
TypeError: Cannot read property 'split' of undefined
```

**Solution:**

Make sure TypeScript/ESLint doesn't complain about the placeholder syntax:

```typescript
// Add type assertion or @ts-ignore if needed
const API_URL = "__VITE_API_URL:http://localhost:3000__" as string;

// Or use environment variable at build time
const API_URL = import.meta.env.VITE_API_URL || "__VITE_API_URL:http://localhost:3000__";
```

---

### Issue: Performance is slow

**Solutions:**

1. **Increase workers:**
```bash
env-mapper -w 8 -d ./dist -p .env.production
```

2. **Limit file types:**
```bash
# Only process JS files if HTML doesn't have placeholders
env-mapper -s js -d ./dist -p .env.production
```

3. **Use output directory instead of overwrite:**
```bash
# Faster for large projects
env-mapper -d ./dist -o ./dist-processed -p .env.production
```

---

## 📊 Advanced Patterns

### Pattern 1: Environment-Specific Defaults

```typescript
// Different defaults for different build modes
const API_URL = process.env.NODE_ENV === 'production'
  ? "__VITE_API_URL:https://api.example.com__"
  : "__VITE_API_URL:http://localhost:3000__";
```

### Pattern 2: Complex JSON Configuration

```typescript
const config = "__VITE_CONFIG:{\"api\":{\"url\":\"http://localhost:3000\",\"timeout\":5000},\"features\":{\"darkMode\":true,\"analytics\":false}}__";

// After parsing
const parsedConfig = JSON.parse(config);
```

### Pattern 3: Conditional Replacement

```bash
# Only replace if environment is production
if [ "$ENV" = "production" ]; then
  env-mapper -d ./dist -p .env.production
fi
```

### Pattern 4: Multi-Stage Replacement

```bash
# Stage 1: Replace common variables
env-mapper -d ./dist -p .env.common

# Stage 2: Replace environment-specific variables
env-mapper -d ./dist -p .env.$ENVIRONMENT
```

---

## 🧪 Testing Guide

### Unit Test Your Config

```typescript
// config.test.ts
import { config } from './config';

describe('Config', () => {
  it('should have valid API URL', () => {
    expect(config.apiUrl).toMatch(/^https?:\/\//);
  });
  
  it('should have numeric timeout', () => {
    expect(typeof config.timeout).toBe('number');
    expect(config.timeout).toBeGreaterThan(0);
  });
});
```

### Integration Test with env-mapper

```bash
#!/bin/bash
# test-env-mapper.sh

set -e

# Build
npm run build

# Test 1: Default values
env-mapper -d ./dist -o ./dist-test1 -p .env.production
grep -q "http://localhost:3000" ./dist-test1/assets/*.js && echo "✅ Test 1 passed"

# Test 2: Custom values
export VITE_API_URL="https://test-api.com"
env-mapper -d ./dist -o ./dist-test2 -p .env.production
grep -q "https://test-api.com" ./dist-test2/assets/*.js && echo "✅ Test 2 passed"

# Test 3: Filtering
env-mapper -d ./dist -o ./dist-test3 -p .env.production --exclude "_SECRET"
! grep -q "VITE_SECRET" ./dist-test3/assets/*.js && echo "✅ Test 3 passed"

echo "All tests passed! 🎉"
```

---

## 📚 Additional Resources

- **GitHub Repository:** https://github.com/TikTzuki/tiktuzki-scripts
- **Examples:** https://github.com/TikTzuki/tiktuzki-scripts/tree/master/examples/env-mapper
- **Issue Tracker:** https://github.com/TikTzuki/tiktuzki-scripts/issues

---

## 🤝 Contributing

Found a bug or have a feature request? Please open an issue on GitHub!

---

## 📄 License

Apache License 2.0 - see LICENSE file for details.

