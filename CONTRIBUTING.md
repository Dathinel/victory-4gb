# Contribute a result from a low-VRAM PC

Have a laptop or desktop with 4–6 GB of dedicated graphics memory? An observation from your own GPU helps us learn where the RTX 3050 Laptop results carry over. Other NVIDIA and AMD cards are welcome. A synthetic shader test, a scene that renders correctly, or a startup failure can each answer a different question.

[Open a GPU report](https://github.com/Dathinel/victory-4gb/issues/new?template=gpu-report.yml) · [Browse existing reports](https://github.com/Dathinel/victory-4gb/issues)

1. Look for your symptom in the existing issues. If your card adds a new data point, reply there with the model and dedicated VRAM. Include Laptop or desktop in the model name.
2. Give the branch and commit if available, and describe what you saw. For a synthetic test, include the command, the result and the adapter (hardware or WARP). A skipped test is worth mentioning.
3. For a performance comparison, keep the scene, resolution and build fixed, change one setting, and report both frame times or FPS. A single observation is welcome too. Share short text excerpts with personal paths removed. Keep game files and assets off the issue tracker.

To try a ready-made fixture, [run the shader test on your GPU](README.md#test-a-shader-on-your-gpu).

For code changes, see the [current priorities and target branches](README.md#contributing).

## Participa en español

¿Tienes una NVIDIA o AMD con poca VRAM? Puedes [enviar el informe en español](https://github.com/Dathinel/victory-4gb/issues/new?template=gpu-report.yml). Indica el modelo exacto, la memoria dedicada y qué viste, incluso si la prueba se detuvo antes de mostrar imagen. Si ya hay un issue sobre ese síntoma, añade allí el resultado de tu tarjeta. También sirven pruebas sintéticas sin el juego: incluye el comando, el resultado y si usaste la tarjeta física o WARP. Si aparece `not run`, incluye esa línea en el informe. [Prueba un shader en tu GPU](README.md#test-a-shader-on-your-gpu).
