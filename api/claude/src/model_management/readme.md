# model_management/

Model listing, recommendation, detail retrieval, and comparison types.

| File | Responsibility |
|------|----------------|
| core.rs | Core model types: ModelInfo, UseCase, ModelRecommendation, filters |
| enhanced.rs | Extended model detail structs: capabilities, pricing, lifecycle |
| enhanced_impls.rs | Implementations for enhanced model detail and comparison methods |
| manager.rs | ModelManager: caching, recommendation engine, and search logic |
