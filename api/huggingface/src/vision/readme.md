# vision

### Purpose

Vision API types and implementations — image classification, detection, and captioning.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — `Vision<E>` type and submodule declarations |
| `types.rs` | Shared vision types — `ImageInput`, bounding boxes, classification labels |
| `classification.rs` | Image classification with confidence scores |
| `detection.rs` | Object detection with bounding boxes |
| `captioning.rs` | Image-to-text caption generation |
