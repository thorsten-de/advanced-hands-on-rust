# Rust workspaces

[Rust Workspaces](Rust%20Workspaces.md) können genutzt werden, um große Projekte zu strukturieren. Das bietet folgende Vorteile:
- Gruppierung verschiedener Komponenten, z.B. Bibliotheken und Anwendungen, die sie nutzen
- Sie teilen sich kompilierte Abhängigkeiten, um  _Zeit_ und _Plattenplatz_ zu sparen
- Cargo kann Aktionen wie `cargo clean` über den gesamten Workspace ausführen.

> [!Info] Projektnamen müssen unterschiedlich sein
> Innerhalb eines Workspaces dürfen keine Projekte mit demselben Namen liegen
