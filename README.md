# Sauvegarde Serveur
Ce projet a été développé dans le cadre d'un TD de continuité de service. Le client à été développé par mon coepéquipé: https://github.com/0xA00/Client-save.

## Fonctionnement
La communication avec le serveur passe par une REST API. Le client communique avec le serveur dans l'objectif de faire une sauvegarde ou restaurer des fichiers.

Lors d'une sauvegarde. Le serveur utilise la fonction de hashage SHA256 sur chaque fichier et enregistre le resultat dans une base de donnée SQLite. Deux fichiers ayant des hash différents, sont différents. De ce fait, le client peut utiliser le hash des fichiers pour savoir lesquels doivent être sauvegardés ou restaurés au lieu de le faire inutilement pour chaque fichier.
