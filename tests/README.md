# Overview
This test suite verifies that all rendering features work correctly by comparing rendered output against golden reference images.

# Test Suites

## `rendering_tests.rs`
Comprehensive test suite with individual tests for each rendering feature:
- **Lambertian Materials** - Tests diffuse material rendering
- **Metal Materials** - Tests metallic surfaces with varying roughness
- **Dielectric Materials** - Tests glass/transparent materials with different IORs
- **Camera Position** - Tests different camera angles and positions
- **Depth of Field** - Tests focal blur effects
- **Complex Scene** - Tests rendering with many mixed materials
- **Shadow Rendering** - Tests shadow generation
- **Performance** - Ensures rendering completes in reasonable time

## Golden Files

Golden reference files are stored in `tests/golden/`:
- Files ending with `.ppm` are the reference images
- Each test has its own golden file (e.g., `lambertian_materials.ppm`)

### Generating Golden Images

Each test in `rendering_tests.rs` has a corresponding `_snapshot` function:
```bash
# Generate golden image for lambertian test
cargo test test_lambertian_materials_snapshot -- --ignored

# Generate all golden images
cargo test --test rendering_tests -- --ignored
```


# Running Tests

```bash
# Run all rendering tests
cargo test --test rendering_tests

# Generate/regenerate golden images for specific tests
cargo test --test rendering_tests -- --ignored --nocapture

# Run a specific test
cargo test --test rendering_tests test_lambertian_materials
```

## Test Configuration

- **Resolution**: 512x512 pixels
- **Frames**: 100 frames for convergence
- **Tolerance**: 2% pixel difference tolerance

## Adding New Tests

1. Create a scene setup function (e.g., `create_my_scene()`)
2. Create the test function that calls `test_scene_against_golden()`
3. Create the snapshot function with `#[ignore]` that calls `generate_golden_image()`
4. Run the snapshot function to generate the golden reference
5. Verify the golden image looks correct
6. Run the test to ensure it passes
