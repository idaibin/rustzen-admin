# UI Feature Index

`DESIGN.md` is the only current shared visual-semantics authority. The files in
this directory are historical task-local mappings, contracts, and evidence from
the prior UI-standardization task; they do not define current token values or
component semantics. Load a target feature contract below only for its
slice-local behavior.

| Product area | UI slice | Product basis | Status |
| --- | --- | --- | --- |
| Admin | [dashboard-navigation-simplification](./features/dashboard-navigation-simplification.md) | [Product foundation](../product/product.md) | Implemented; post-merge browser review pending |
| Admin | [role-definition-management](./features/role-definition-management.md) | [Product spec](../product/features/role-definition-management/spec.md) | Implemented; pre-merge runtime and browser evidence retained |
| Admin | [user-role-assignment-readiness](./features/user-role-assignment-readiness.md) | [Product spec](../product/features/user-role-assignment-readiness/spec.md) | Implemented; pre-merge runtime and browser evidence retained |

These contracts reuse the current visual system. The Dashboard slice updates
the global route inventory and structured manifest; the role and user slices do
not create a new design-system revision.
