watch-with-test:
	RUST_LOG=debug cargo watch -x check -x test -x run

docker-build:
	docker build --tag zero2prod --file docker/app/Dockerfile .