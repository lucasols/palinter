build:
	cargo build --release
	cp target/release/palinter npm/bin
	cd npm && pnpm build &&	pnpm publish


