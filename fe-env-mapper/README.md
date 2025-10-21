

## Front end Vite

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

3. Run script

Dev mode:
```shell
cargo run --color=always --bin fe-env-mapper -- \
-d examples/env-mapper/js_source_code/dist \
-e examples/env-mapper/dynamic.env
```

Production mode:
```shell
map_env \
-d examples/env-mapper/js_source_code/dist \
-e examples/env-mapper/dynamic.env
```