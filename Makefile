# migrate-up:
# 	sqlx migrate run
# migrate-down:
# 	sqlx migrate revert
dev:
	cargo watch -q -c -w src/ -x run