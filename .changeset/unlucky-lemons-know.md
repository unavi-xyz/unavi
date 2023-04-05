---
"client": patch
---

Can now run the client without needing S3 storage or a database. Features that rely on those services (such as the editor) will be removed from the UI if the needed environment variables are not present.
