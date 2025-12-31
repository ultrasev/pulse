.PHONY: dev build install clean

dev:
	bun tauri dev

build:
	bun tauri build

install:
	@echo "Installing to /Applications..."
	cp -r tauri/target/release/bundle/macos/Pulse.app /Applications/
	@echo "Installation complete!"

clean:
	rm -rf dist
	rm -rf tauri/target
