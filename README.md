# Ray Tracing in a Weekend (in Rust)
This was a quick (10hrs~) implementation of the concepts presented in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html). The code isn't a direct translation, some decisions were taken to make the code more idiomatic, and others were taken for performance reasons[^1].

# Future work
- Work on GPU support
- Add support for triangles
  - More generally add support for more 3D objects
- Make it more efficient
  - e.g. more parallelization
- Actually use light sources


[^1]: For example the use of Rayon for easy parallelization of sampling

