#[tauri::command]
pub fn get_snippets() -> Result<Vec<(String, String)>, String> {
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

    let snippet_dir = PathBuf::from("snippets");
    let mut snippets = Vec::new();

    if snippet_dir.exists() && snippet_dir.is_dir() {
        for entry in fs::read_dir(snippet_dir).map_err(|e| e.to_string())? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)
                            .map_err(|e| e.to_string())?;
                        snippets.push((name.to_string(), contents));
                    }
                }
            }
        }
    }

    Ok(snippets)
}
