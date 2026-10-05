help:
  just --list

watch-ui:
  cargo watch -q -c -w "ui" \
    -x "run -p feiri-ui"
