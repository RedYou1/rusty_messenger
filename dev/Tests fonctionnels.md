# API
le cargo test de l'api fonctionne (il fait les tests du client automatiquement)

# Client
lorsque l'api ne répond pas au client -> message d'erreur "Perte de connection"
lorsque le client perd la connection avec l'api (Event Source) -> redirection a la Connection d'utilisateur
lorsque l'utilisateur réussi une action -> retire son message d'erreur, s'il y a un.

## Connection d'utilisateur
aucun champ de rempli -> message d'erreur "Il faut au moins une lettre dans le nom"
aucune lettre dans le champ mot de passe -> message d'erreur "Il faut au moins une lettre dans le mot de passe"
avec un mauvais identifiant ou mot de passe -> message d'erreur "Mauvais identifiant ou mot de passe"
bon identifiant et mot de passe -> redirection a la liste des salons
cliquez sur "Création d'utilisateur" -> redirection a la Création d'utilisateur

## Creation d'utilisateur
aucun champ de rempli -> message d'erreur "Il faut au moins une lettre dans le nom"
aucune lettre dans le champ mot de passe -> message d'erreur "Il faut au moins une lettre dans le mot de passe"
identifiant déjà pris -> message d'erreur "Identifiant déjà pris"
bon identifiant et mot de passe -> redirection a la liste des salons
cliquez sur "Connection d'utilisateur" -> redirection a la Connection d'utilisateur

## liste des salons
aucun champ de rempli -> message d'erreur "Il faut au moins une lettre dans le nom du salon"
bon nom de salon -> ajoute le salon à la liste et vide le champ
cliquez sur un salon -> redirection a la page du salon

## salon
cliquez sur < en haut à gauche -> redirection a la liste des salons
aucune lettre dans le champ d'invitation -> message d'erreur "Il faut au moins une lettre dans le nom"
utilisateur déjà dans le salon -> message d'erreur "Cet utilisateur est déjà dans ce salon."
utilisateur inexistant -> message d'erreur "Pas d'utilisateur avec ce nom <le nom>"
bonne invitation -> l'utilisateur est ajouté et vide le champ
aucune lettre dans le champ de message -> message d'erreur "Il faut au moins une lettre dans le message"
bon message -> le message est envoyer et vide le champ