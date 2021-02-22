# Extended-NFT
## Principes
Le principe des Extended-Non-Fongible-Token est détendre les fonctions des NFT en ajoutant 
* la gestion d'un prix de vente et les reversements associés,
* la gestion des revendeurs et le versement des commissions associées (selon le principe d'un mandat de vente),
* la gestion d'un statut "en vente" permettant l'implémentation d'une marketplace,
* la possibilité de contenir de l'information uniquement destinée au propriétaire sous la forme d'un secret

Le fonctionnement général repose sur 3 types de profil :
* Le "mineur", qui fabrique le token
* le "distributeur", qui dispose d'un mandat pour vendre le token sur sa propre place de marché,
* le "propriétaire" du token
    
Les profils sont cumulables. Ainsi par exemple à la création du token, le "mineur" est également le "propriétaire".  
Les distributeurs du token n'en sont pas propriétaires. Il dispose d'une autorisation (d'un mandat) qui est attribué par le propriétaire
Lorsqu'un token est acheté, le nouveau propriétaire peut 
* modifier la liste des distributeurs autorisés et remettre le token en vente,
* l'offrir à quelqu'un
    
## Description d'un token
Chaque token dispose de plusieurs caractéristiques régissant son fonctionnement:
* **price** est son prix initial, fixé par le mineur à sa création,
* **uri** est une description publique du token, affichée dans la marketplace (le contenu peut désigner ressource IPFS),
* **secret** est un texte libre ou un lien IPFS visible uniquement par le propriétaire du token
* **state** est à 0 si le token est en vente, 1 sinon
* **dealer_addr** contient la liste des distributeurs autorisé à vendre le token
* **dealer_markup** contient la commission de chaque distributeur
* **min_markup** & **max_markup** fixe les limites des commissions que les distributeurs peuvent fixer
* **owner** contient l'adresse Elrond du propriétaire du token
* **miner** contient l'adresse Elrond du mineur (créateur) du token
* **seller_owner** indique si le propriétaire est autorisé à mettre en vente et / ou transférer son token,
* **miner_ratio** contient un pourcentage que percoit, à chaque vente, le mineur sur la commission du vendeur    

## Usages
Le SmartContract expose plusieurs méthodes pouvant être appelé par le distributeur, le propriétaire ou le mineur du token suivant la méthode considéré :
  
* **mint** : permet au mineur de fabriquer le token. La fabrication comprend notamment
l'initialisation de certaines propriétés importantes comme le prix initial ou l'encadrement de la commission des distributeurs

* **get_tokens** : est appelé par le distributeur pour récupérer l'ensemble des tokens qu'il a la possibilité de vendre (ie
pour lesquels il a été déclarer comme vendeur par le propriétaire). La méthode retourne une suite d'octets qui doit être découpé
précisément pour récupérer les caractéristiques publiques de chaque token : prix, libellés, adresse du mineur, du propriétaire notamment
 
* **buy** : est appelé par le vendeur avec la clé privée de l'acheteur et le montant de l'achat. Le vendeur transmet son adresse
pour recevoir sa commission. L'ancien propriétaire recoit le prix initial et les vendeur et mineur sont rétribués suivant 
*miner_ratio* et *dealer_markup*

* **setstate** permet au propriétaire de mettre en vente / retirer de la vente le token qu'il posséde

* **add_dealer** permet au propriétaire d'ajouter un distributeur au token
* **clear_dealer** retire l'ensemble des distributeurs

* **price** est appelé par le distributeur pour modifier sa commission (dans l'interval fixé à la création du token par le mineur)

* **open** est appelé par le propriétaire pour dévoiler le secret contenu dans le token

* **burn** est appelé par le mineur pour détruire un token

## Remarques
### Rétribution du propriétaire
Le prix initial fixé à la création du token n'est jamais modifiable. 
il détermine ce que reçoit le propriétaire lorsqu'il vend le token. 
Si ce dernier souhaite faire une marge, il doit également se déclarer 
comme distributeur et fixer la commission qu'il souhaite recevoir


## Evolutions prévues
ENFT est encore en construction. Certaines évolutions doivent être mise en oeuvre avant d'être en version béta :
* le secret doit être crypté à la fabrication du token par une clé connu du mineur et du contrat, 
sinon il reste visible dans l'explorateur par analyse de la transaction
* certaines fonctions doivent être ajoutée afin de garantir la compatibilité avec ERC721
* quelques optimisations sont à étudier notamment autour de la consommation de gas à l'achat des tokens : sortir le propriétaire de la structure
du token pour miniser l'impact d'un changement de propriétaire (vente et transfert du token) 
* permettre l'usage d'ESDT à la place de l'eGold
* mettre en place la possibilité d'une destruction automatique du token après ouverture


on s'appuit sur l'exemple https://github.com/ElrondNetwork/elrond-wasm-rs/tree/master/contracts/examples/non-fungible-tokens
