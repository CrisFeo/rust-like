list:
  just -l -u

watch:
  bacon clippy

dev:
  just run || :
  just logs

run:
  RUST_BACKTRACE=1 cargo run --color always 2> logs.txt

logs:
  less -R logs.txt
