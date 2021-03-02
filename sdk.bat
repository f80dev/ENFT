#Lancement de l'environnement elrond-dev sous windows (via usage d'une image docker)

docker rm -f elrond-dev
docker run --name elrond-dev -v C:/Users/hhoareau/CLionProjects/ENFT/:/home/root/dev/ -ti f80hub/elrond-dev bash