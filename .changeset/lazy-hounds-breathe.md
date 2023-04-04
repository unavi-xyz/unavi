---
"client": minor
---

Refactor project and publication storage. May break existing projects and spaces. Will allow for better caching behavior, as published models are now immutable. When updates are pushed to a publication, a unique S3 path will be created to store the model.
