
## Install
```shell
curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
```

## Map Vite .env

1. Using env

React env `.env.production`:
```dotenv
VITE_API_URL=\${VITE_API_URL}
```

HTML Constant Replacement:
```html
<h1>My sample ENV: %VITE_API_URL%</h1>
```

2. Build command inside `package.json`:
```json
{
  "build": "tsc -b && vite build --mode production"
}
```
```shell
npm run build
```

3. Run script
```shell
Usage: fe-env-mapper [OPTIONS]

Options:
  -d, --dir <DIR>
          [default: /js_source_code/dist]

  -p, --production-env-file <PRODUCTION_ENV_FILE>
          env value: VITE_API_URL=\${VITE_API_URL}
          [default: .env.production]

  -e, --dynamic-env-file <DYNAMIC_ENV_FILE>
          Dynamic env value file to override current envs, default: None

  -s, --suffixes <SUFFIXES>
          [default: js,html]

  -o, --output-dir <OUTPUT_DIR>
          Output directory for processed files, default: overwrite

  -w, --worker <WORKER>
          Number of parallel workers
          [default: 1]

  -h, --help
          Print help (see a summary with '-h')
```

```shell
./map_env \
-d examples/env-mapper/js_source_code/dist \
-e examples/env-mapper/js_source_code/.env.production
```

## Releases script
```shell
cargo build --release --bin fe-env-mapper --target x86_64-apple-darwin && cp target/x86_64-apple-darwin/release/fe-env-mapper release/macos_amd/map_env 
cargo build --release --bin fe-env-mapper --target aarch64-apple-darwin && cp target/aarch64-apple-darwin/release/fe-env-mapper release/macos_arm/map_env
cargo build --release --bin fe-env-mapper --target x86_64-unknown-linux-gnu && cp target/x86_64-unknown-linux-gnu/release/fe-env-mapper release/linux_amd/map_env
cargo build --release --bin fe-env-mapper --target aarch64-unknown-linux-gnu && cp target/aarch64-unknown-linux-gnu/release/fe-env-mapper release/linux_arm/map_env
```