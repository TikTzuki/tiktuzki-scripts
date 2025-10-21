
```shell
npm run build
npx dotenv -e .env.production -- tsc -b && npx dotenv -e .env.production -- vite build
npx serve -s dist
```