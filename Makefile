PREFIX = /usr

install: 
	cargo build --release
	sudo cp -f ./target/release/sid-bg ${PREFIX}/bin
	sudo chmod 755 ${PREFIX}/bin/sid-bg
	sudo cp apod.service /etc/systemd/user/
	sudo systemctl daemon-reload