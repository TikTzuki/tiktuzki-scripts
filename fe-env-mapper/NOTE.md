Dev mode:
```shell
cargo run --color=always --bin fe-env-mapper -- \
-d examples/env-mapper/js_source_code/dist \
-p examples/env-mapper/js_source_code/.env.production \ 
-o fe_output
```