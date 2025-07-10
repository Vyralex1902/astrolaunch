use std::path::Path;
use strsim;
use walkdir::WalkDir;

#[tauri::command]
pub fn search_files(query: &str) -> Result<Vec<String>, String> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    #[cfg(target_os = "macos")]
    let search_paths = vec![Path::new("/")];

    #[cfg(target_os = "windows")]
    let search_paths = vec![Path::new("C:\\"), Path::new("D:\\"), Path::new("E:\\")];

    let mut heap = BinaryHeap::new();
    let mut seen = std::collections::HashSet::new();

    for root in search_paths {
        let walker = WalkDir::new(root).into_iter();
        for entry in walker.filter_map(|e| e.ok()).take(100_000) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
                    let score = strsim::jaro_winkler(query, name);
                    if score > 0.6 && seen.insert(path.to_path_buf()) {
                        heap.push(Reverse((
                            (score * 10000.0) as i32,
                            path.display().to_string(),
                        )));
                        if heap.len() > 8 {
                            heap.pop();
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<_> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, path))| path)
        .collect();

    results.reverse(); // highest score first
    Ok(results)
}
