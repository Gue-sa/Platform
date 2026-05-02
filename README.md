Ce projet est destiné à modéliser une plateforme maritime autonome (capitainerie + bateaux). Sont implémentés : la communication via AIS, une simulation de GPS et de communications satellite. La capitainerie est capable de générer et d'attribuer des ordres de voyage aux navires disponibles que ces derniers exécuteront.

Attention, ce projet est destiné à tourner sur Linux. Windows n'est pas supporté et je n'ai pas testé avec WSL.

Comment lancer le projet ?

- Compiler pour son architecture / OS.
- Lancer le launcher.
- Lancer les différents composants.
- Si la capitainerie est lancée, l'interface armateur est disponible à l'url suivante : http://localhost:3000. L'API l'est à : http://localhost:8000

ATTENTION : si vous souhaitez lancer une simulation complète, la séquence de lancement est la suivante : serveur, capitainerie puis bateau.
