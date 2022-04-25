build: build-frontend
	cargo build --release
	@cp target/release/libreads ./release
	@echo 'built into ./release/'

build-frontend:
	@rm -rf release
	@mkdir -p release/frontend
	cd frontend && npm run build
	@cp -r frontend/build ./release/frontend/

