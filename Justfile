nothing:

build:
    cargo build --release --target arm-unknown-linux-gnueabihf --features rpi

install: build
    ansible-playbook -i inventory.yml deploy.yml -t install --diff  
