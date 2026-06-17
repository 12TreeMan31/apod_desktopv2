PREFIX = /usr

run:
	cargo run -- -c config

install: 
	cargo build --release
	sudo cp -f ./target/release/adi-bg ${PREFIX}/bin
	sudo chmod 755 ${PREFIX}/bin/adi-bg
	sudo cp apod.service /etc/systemd/user/
	sudo systemctl daemon-reload