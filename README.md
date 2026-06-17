# FLIP

FLIP simulation made in rust. Based on work of Matthias Müller

https://github.com/matthias-research

https://matthias-research.github.io/pages/tenMinutePhysics/index.html

UWAGA: Program należy kompilować z flagą `--release`, w przeciwnym wypadku simulation loop jest dosyć powolny (pełne 60 FPS w release mode vs około 3 FPS w debug mode).

Jak uruchomić:
`cargo run --release`

Parametry symulacji można edytować w `src/main.rs`.
