PREFIX = /usr

install: 
	cargo build --release
	sudo cp -f ./target/release/apod_desktopv2 ${PREFIX}/bin
	sudo chmod 755 ${PREFIX}/bin/apod_desktopv2
	sudo cp apod.service /etc/systemd/user/
	sudo systemctl daemon-reload