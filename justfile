run:
    cargo run

run-with-logs-widget:
    TOOT_RS_SHOW_LOGS=true cargo run

logs:
    jq -C . ~/.local/state/toot-rs/toot-rs.json.* | bat

bat-logs:
    bat ~/.local/state/toot-rs/toot-rs.json.* --wrap=never --language=json

rm-logs:
    rm -rf  ~/.local/state/toot-rs/toot-rs.json.*

