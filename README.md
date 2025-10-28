# TikTzuki Scripts


## FE Env Mapper

Install
```shell
curl https://raw.githubusercontent.com/TikTzuki/tiktuzki-scripts/refs/heads/master/install.sh | bash
```
Examples usage: https://github.com/TikTzuki/tiktuzki-scripts/tree/master/examples/env-mapper
```shell
Usage: env-mapper [OPTIONS]

Options:
  -d, --dir <DIR>
          [default: /js_source_code/dist]
  -p, --production-env-file <PRODUCTION_ENV_FILE>
          env value: VITE_API_URL=__VITE_API_URL__ [default: .env.production]
      --placeholder <PLACEHOLDER>
          Template place holder: 1. __KEY__ 
           2. {{KEY}} 
           3. ${KEY} 
           4. ${{KEY}} [default: 1]
  -e, --runtime-env-file <RUNTIME_ENV_FILE>
          Runtime env value file to override current envs, default: None
  -s, --suffixes <SUFFIXES>
          [default: js,html]
  -o, --output-dir <OUTPUT_DIR>
          Output directory for processed files, default: overwrite
  -w, --worker <WORKER>
          Number of parallel workers [default: 1]
  -h, --help
          Print help (see more with '--help')
```
