.PHONY: dev build install clean bump release

dev:
	bun tauri dev

build:
	bun tauri build --bundles app
	@echo "Built: tauri/target/release/bundle/macos/Pulse.app"

install:
	@echo "Installing to /Applications..."
	cp -r tauri/target/release/bundle/macos/Pulse.app /Applications/
	@echo "Installation complete!"

clean:
	rm -rf dist
	rm -rf tauri/target

# Usage: make bump v=0.0.3
bump:
	@if [ -z "$(v)" ]; then echo "Usage: make bump v=0.0.3"; exit 1; fi
	@echo "Bumping version to $(v)..."
	@sed -i '' 's/"version": "[^"]*"/"version": "$(v)"/' package.json
	@sed -i '' 's/"version": "[^"]*"/"version": "$(v)"/' tauri/tauri.conf.json
	@sed -i '' 's/>v[0-9]*\.[0-9]*\.[0-9]*</>v$(v)</' app/components/App.svelte
	@sed -i '' 's/version "[^"]*"/version "$(v)"/' homebrew-tap-template/Casks/pulse.rb
	@echo "Version updated to $(v)"

# Usage: make release v=0.0.3
release: bump
	git add -A
	git commit -m "chore: bump version to $(v)"
	git tag v$(v)
	git push origin master
	git push origin v$(v)
	@echo "Released v$(v)!"
