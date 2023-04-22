run:
    cargo run

bat-logs:
    bat ~/.local/state/toot-rs/log.json.* --wrap=never --language=json

jq-logs:
    jq -C . ~/.local/state/toot-rs/log.json.* | bat

rm-logs:
    rm -rf  ~/.local/state/toot-rs/log.json}.*
