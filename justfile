logs:
    # bat ~/.local/state/toot-rs/toot-rs.log.*
    jq -C . ~/.local/state/toot-rs/toot-rs.log.* | bat

rm-logs:
    rm -rf  ~/.local/state/toot-rs/toot-rs.log.*
