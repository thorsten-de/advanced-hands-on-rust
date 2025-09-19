# Entity Component System
#ECS

Ein ECS ist eine schnelle spielinterne Datenbank, die den aktuellen Zustand des Spiels beschreibt. Sie ermöglicht die Definition von unabhängigen Komponenten, die zu [Entität](Entität.md) _komponiert_ werden, um Funktionalitäten des Systems zu beschreiben.

- Eine [[Entität]] ist kaum mehr als ein _Identifikator_ für etwas. Sie werden durch [[Komponente]] beschrieben.
- Eine [Komponente](Komponente.md) ist eine Informationseinheit, die für verschiedene Dinge verwendet werden kann. Eine `Position` kann z.B. von einem `Character` genutzt werden sowie weiteren Dingen, die auf einer Karte platziert werden
- Ein [[System]] nutzt die Informationen aus der ECS-Datenbank und steuert logisch den Spielverlauf.
- [[Nachrichten]] werden zwischen Systemen ausgetauscht, um deren Ergebnisse zu nutzen
- [[Ressourcen]] sind global geteilte Daten des Programms

Siehe Bild S.8