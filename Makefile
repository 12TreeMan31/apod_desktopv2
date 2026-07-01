PREFIX = /usr

install: 
	cargo build --release
	sudo cp -f ./target/release/sid-bg ${PREFIX}/bin
	sudo chmod 755 ${PREFIX}/bin/sid-bg
	sudo cp ./systemd/* /etc/systemd/user/
	sudo systemctl daemon-reload

uninstall:
	cargo clean
	sudo rm ${PREFIX}/bin/sid-bg