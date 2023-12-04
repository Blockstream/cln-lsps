use std::env;
use std::path::Path;

// generated by `sqlx migrate build-script`
fn main() {
    // The sqlx-crate checks queries at compile time against the database schema
    // It will use the database specified in the DATABASE_URL environment variable.
    let initial_db_url = env::var("DATABASE_URL");
    match initial_db_url {
        Ok(_) => {} // Don't change the user specified config
        Err(_) => {
            // By default we use the path "./data/lsp_server.db" to initiate the database
            let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let data_path = Path::new(&cargo_manifest_dir).join("data");
            let db_file_path = Path::new(&data_path).join("lsp_server.db");
            let db_url = format!("sqlite:{}", db_file_path.to_str().unwrap());

            // If the paths doesn't exist yet we'll create it and ensure a database exists
            if !data_path.exists() {
                std::fs::create_dir(&data_path).unwrap();
            }

            // This command ensures the build-process uses this environment variable
            println!("cargo:rustc-env=DATABASE_URL={}", db_url);
        }
    }

    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
}
