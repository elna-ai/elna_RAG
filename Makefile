shell:
	./toolchain.sh bash

deploy_local:
	./toolchain.sh dfx start --background
	./toolchain.sh dfx deploy

start:
	./toolchain.sh dfx start
