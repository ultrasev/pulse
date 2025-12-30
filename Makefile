.PHONY: dev build install clean

dev:
	bun tauri dev

build:
	bun tauri build

install:
	@echo "Installing to /Applications..."
	cp -r src-tauri/target/release/bundle/macos/system-monitor.app /Applications/
	@echo "Installation complete!"

clean:
	rm -rf dist
	rm -rf src-tauri/target
