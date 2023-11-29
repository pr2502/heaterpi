nothing:

build:
    cargo build --release --target arm-unknown-linux-gnueabihf --features rpi
    arm-linux-gnueabihf-strip -s target/arm-unknown-linux-gnueabihf/release/heaterpi


install: build
    ansible-playbook -i inventory.yml deploy.yml -t install --diff  
