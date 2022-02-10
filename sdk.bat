#Lancement de l'environnement elrond-dev sous windows (via usage d'une image docker)
# docker rm -f elrond-dev && docker run --name elrond-dev -v C:/Users/hhoar/CLionProjects/elrond-wasm-rs/:/home/root/sample/ -v C:/Users/hhoar/CLionProjects/ENFT/:/home/root/dev/ -v C:/Users/hhoar/CLionProjects/elrond-wasm-rs/:/home/root/samples/ -ti f80hub/elrond-dev bash
docker rm -f elrond-dev
docker run --name elrond-dev -v C:/Users/hhoar/CLionProjects/elrond-wasm-rs/:/home/root/sample/ -v C:/Users/hhoar/CLionProjects/ENFT/:/home/root/dev/ -v C:/Users/hhoar/CLionProjects/elrond-wasm-rs/:/home/root/samples/ -ti f80hub/elrond-dev bash
