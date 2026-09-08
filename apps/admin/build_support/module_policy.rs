const SOURCE_FILES: &[&str] = &[
    "apps/web/src/api/module-contract.ts",
    "apps/web/src/api/request.ts",
    "apps/web/src/api/runtime.ts",
    "apps/web/src/components/language-switch.tsx",
    "apps/web/src/components/theme-provider.tsx",
];
const SOURCE_DIRECTORIES: &[&str] = &[
    "apps/web/src/api/installation/",
    "apps/web/src/api/monitor/",
    "apps/web/src/assets/",
    "apps/web/src/components/auth/",
    "apps/web/src/components/feedback/",
    "apps/web/src/components/page/",
    "apps/web/src/components/table/",
    "apps/web/src/components/user/",
    "apps/web/src/constant/",
    "apps/web/src/hooks/",
    "apps/web/src/lib/",
];
const PACKAGES: &[&str] = &[
    "@ant-design/colors",
    "@ant-design/cssinjs",
    "@ant-design/cssinjs-utils",
    "@ant-design/fast-color",
    "@ant-design/icons",
    "@ant-design/icons-svg",
    "@ant-design/pro-components",
    "@ant-design/react-slick",
    "@babel/runtime",
    "@ctrl/tinycolor",
    "@dnd-kit/accessibility",
    "@dnd-kit/core",
    "@dnd-kit/modifiers",
    "@dnd-kit/sortable",
    "@dnd-kit/utilities",
    "@emotion/hash",
    "@emotion/unitless",
    "@reduxjs/toolkit",
    "@tanstack/history",
    "@tanstack/query-core",
    "@tanstack/react-query",
    "@tanstack/react-router",
    "@tanstack/react-store",
    "@tanstack/store",
    "@umijs/route-utils",
    "antd",
    "clsx",
    "cn",
    "compute-scroll-into-view",
    "d3-array",
    "d3-color",
    "d3-format",
    "d3-interpolate",
    "d3-path",
    "d3-scale",
    "d3-shape",
    "d3-time",
    "d3-time-format",
    "dayjs",
    "decimal.js-light",
    "dequal",
    "es-toolkit",
    "eventemitter3",
    "immer",
    "internmap",
    "is-mobile",
    "json2mq",
    "lodash-es",
    "path-to-regexp",
    "react",
    "react-dom",
    "react-is",
    "react-redux",
    "recharts",
    "redux",
    "redux-thunk",
    "reselect",
    "safe-stable-stringify",
    "scheduler",
    "scroll-into-view-if-needed",
    "seroval",
    "seroval-plugins",
    "string-convert",
    "stylis",
    "swr",
    "throttle-debounce",
    "tiny-invariant",
    "use-sync-external-store",
    "victory-vendor",
    "zustand",
];
const RC_COMPONENTS: &[&str] = &[
    "async-validator",
    "cascader",
    "checkbox",
    "collapse",
    "color-picker",
    "context",
    "dialog",
    "drawer",
    "dropdown",
    "form",
    "image",
    "input",
    "input-number",
    "mentions",
    "menu",
    "mini-decimal",
    "motion",
    "mutate-observer",
    "notification",
    "overflow",
    "pagination",
    "picker",
    "portal",
    "progress",
    "qrcode",
    "rate",
    "resize-observer",
    "segmented",
    "select",
    "slider",
    "steps",
    "switch",
    "table",
    "tabs",
    "tooltip",
    "tour",
    "tree",
    "tree-select",
    "trigger",
    "upload",
    "util",
    "virtual-list",
];

pub fn validate(module_ids: &[&str], generated_prefix: &str) {
    let mut generated = 0;
    for raw in module_ids {
        let id = raw.split_once('?').map_or(*raw, |(path, _)| path);
        assert!(safe_id(id), "selected Web module inventory contains an unsafe module ID: {raw}");
        if id.starts_with('\0') {
            assert!(
                id == "\0vite/preload-helper.js" || id == "\0vite/modulepreload-polyfill.js",
                "selected Web module inventory contains an unknown virtual ID: {raw}"
            );
        } else if id.starts_with(generated_prefix) {
            generated += 1;
        } else if id.starts_with("apps/web/.selected-web/") {
            panic!("selected Web module inventory contains another composition source: {raw}");
        } else if SOURCE_FILES.contains(&id)
            || SOURCE_DIRECTORIES.iter().any(|directory| id.starts_with(directory))
        {
        } else if let Some(rest) = id.strip_prefix("apps/web/node_modules/") {
            let package = package_name(rest);
            assert!(allowed_package(package), "unknown selected Web package: {raw}");
        } else {
            panic!("selected Web module inventory contains an unclassified owner: {raw}");
        }
    }
    assert!(generated > 0, "selected Web module inventory has no generated route source");
    for required in [
        "apps/web/src/api/installation/api.ts",
        "apps/web/src/api/monitor/api.ts",
        "apps/web/src/api/request.ts",
    ] {
        assert!(
            module_ids
                .iter()
                .any(|raw| raw.split_once('?').map_or(*raw, |(path, _)| path) == required),
            "selected Web module inventory is missing required owner: {required}"
        );
    }
}

fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('/')
        && !id.starts_with('.')
        && !id.contains('\\')
        && id.split('/').all(|part| part != "..")
}

fn package_name(path: &str) -> &str {
    if path.starts_with('@') {
        let second = path.match_indices('/').nth(1).map_or(path.len(), |(index, _)| index);
        &path[..second]
    } else {
        path.split('/').next().unwrap_or("")
    }
}

fn allowed_package(package: &str) -> bool {
    PACKAGES.contains(&package)
        || package.strip_prefix("@rc-component/").is_some_and(|name| RC_COMPONENTS.contains(&name))
}
