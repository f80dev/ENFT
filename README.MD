#Extended-NFT
##Principes
Le principe des Extended-Non-Fongible-Token est détendre les fonctions des NFT en ajoutant 
    - la gestion d'un prix
    - la gestion des revendeurs et les commissions associées
    - la gestion d'un statut "en vente" permettant l'implémentation d'une marketplace
    - la possibilité de contenir de l'information uniquement destinée au propriétaire : secret

Cette gestion repose sur 3 types de comptes :
    - Le "mineur", qui fabrique le token
    - le distributeur, qui peut vendre le token
    - le propriétaire du token
    
Les 3 rôles sont cumulables. Ainsi le mineur et de fait le premier propriétaire et il peut se déclarer comme vendeur.
    
##Usages




on s'appuit sur l'exemple https://github.com/ElrondNetwork/elrond-wasm-rs/tree/master/contracts/examples/non-fungible-tokens
