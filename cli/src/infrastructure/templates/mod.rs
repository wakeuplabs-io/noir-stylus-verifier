#![cfg_attr(rustfmt, rustfmt::skip)]

pub(crate) const VERIFIER_TEMPLATES: &[(&str, &str)] = &[
    ("src/main.rs", include_str!("verifier/src/main.rs.tera")),
    ("src/lib.rs", include_str!("verifier/src/lib.rs.tera")),
    (".gitignore", include_str!("verifier/.gitignore.tera")),
    ("Cargo.toml", include_str!("verifier/Cargo.toml.tera")),
    ("Cargo.lock", include_str!("verifier/Cargo.lock.tera")),
    ("rust-toolchain.toml", include_str!("verifier/rust-toolchain.toml.tera")),
];

pub(crate) const PROJECT_TEMPLATES: &[(&str, &str)] = &[
    ("src/main.nr", include_str!("project/src/main.nr.tera")),
    ("Nargo.toml", include_str!("project/Nargo.toml.tera")),
    ("Prover.toml", include_str!("project/Prover.toml.tera")),
    ("scripts/verify-global.js", include_str!("project/scripts/verify-global.js.tera")),
    ("scripts/verify.js", include_str!("project/scripts/verify.js.tera")),
    (".gitignore", include_str!("project/.gitignore.tera")),
    ("package.json", include_str!("project/package.json.tera")),
    ("pnpm-lock.yaml", include_str!("project/pnpm-lock.yaml.tera")),
    ("README.md", include_str!("project/README.md.tera")),
];
