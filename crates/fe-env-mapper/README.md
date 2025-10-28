## Map Vite .env

### Using env

Key using [env variables](https://vite.dev/guide/env-and-mode.html) as usual:

```javascript
// React code:
function App() {
    const [_, setCount] = useState(0);
    const apiUrl = import.meta.env.VITE_API_URL;

// Html code:
<h1>My sample ENV: %VITE_API_URL%</h1>
```

Config `.env.production` with template value:
```dotenv
VITE_API_URL=__VITE_API_URL__
```

Run build with production mode:
```shell
cd js_source_code && npx tsc -b && npx vite build --mode production
```

After build, you will see in:

```javascript
// Html file:
<p>__VITE_API_URL__</p>

// JavaScript file:
... children:["Var: ","__VITE_API_URL__"]})}),nt.jsx ...
```

### Run script

[Install](https://github.com/TikTzuki/tiktuzki-scripts/tree/master) env-mapper and run it:


```shell
Usage: map_env [OPTIONS]

```

```shell
./env-mapper-darwin-arm64 \
-d js_source_code/dist \
-e js_source_code/.env.runtime \
-p js_source_code/.env.production \
-o out
```
