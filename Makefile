icon:
	pnpm tauri icon  # ./app-icon.png

test-ask:
	cd src-tauri && cargo test -- test_ask --nocapture
