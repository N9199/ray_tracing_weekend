# Ray Tracing in Rust
Originally based on [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html), now mostly a place to experiment with ideas about performance and new features.

# Rendering scenes

Choose a built-in scene by name, or load a custom scene from a TOML file. Image dimensions and sampling settings remain in the root `Config.toml`; `--output` selects the output PPM path.

```sh
cargo run -p bin -- cornell-box --output image.ppm
cargo run -p bin -- --scene-file scenes/examples/cornell_box.toml --output custom.ppm
```

A scene file has a required `[camera]` section, named `[materials.<name>]` tables, and `[[objects]]` entries. Objects reference material names and have a nested `[objects.shape]` table. Supported shapes are `sphere`, `plane`, `quad`, and `cuboid`; material types are `lambertian`, `metal`, `dielectric`, and `diffuse_light`. Lambertian and diffuse-light materials accept `solid`, `checker`, or `noise` textures. Colors are three-component linear RGB arrays.

Transforms are optional ordered inline-table entries on an object. Translation offsets and rotation degrees are applied in the listed order. Emissive geometry is visible whether or not it is sampled; set `sample_as_light = true` to register a sphere or quad with the light sampler. See [`scenes/examples/cornell_box.toml`](scenes/examples/cornell_box.toml) for a complete custom scene.

Camera `look_from` and `look_at` are required. `vfov` defaults to 40 degrees, `background` to black, `vup` to `[0, 1, 0]`, `defocus_angle` to zero, and `focus_distance` to the distance from `look_from` to `look_at`.

# Future work
- Work on GPU support
- Add support for triangles
  - More generally add support for more 3D objects
- Make it more efficient
