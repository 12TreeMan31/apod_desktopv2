PREFIX = /usr

run:
	mkdir -p /tmp/apod_images/
	mkdir -p /tmp/apod_state/
	cargo run -- -c config

install: 
	cargo build --release
	sudo cp -f ./target/release/adi-bg ${PREFIX}/bin
	sudo chmod 755 ${PREFIX}/bin/adi-bg
	sudo cp apod.service /etc/systemd/user/
	sudo systemctl daemon-reload