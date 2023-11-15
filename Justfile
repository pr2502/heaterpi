nothing:

build:
    cargo build --release

install: build
    ansible-playbook -i inventory.yml deploy.yml -t install --diff  
