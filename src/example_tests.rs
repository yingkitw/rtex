#[cfg(test)]
mod example_tests {
    use std::fs;
    use std::path::Path;
    use crate::NativeTexConverter;

    #[test]
    fn test_convert_all_examples() {
        let examples_dir = Path::new("examples");
        let output_dir = Path::new("output");
        
        // Create output directory
        fs::create_dir_all(output_dir).unwrap();
        
        // Find all .tex files
        let tex_files = fs::read_dir(examples_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "tex") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        
        assert!(!tex_files.is_empty(), "No .tex files found in examples directory");
        
        let mut success_count = 0;
        
        for tex_file in &tex_files {
            let pdf_path = output_dir.join(tex_file.file_name().unwrap()).with_extension("pdf");
            
            let result = NativeTexConverter::convert_file(tex_file, &pdf_path);
            
            match result {
                Ok(_) => {
                    assert!(pdf_path.exists(), "PDF file was not created");
                    let size = fs::metadata(&pdf_path).unwrap().len();
                    assert!(size > 500, "PDF file is too small: {} bytes", size);
                    println!("✓ {}: {} bytes", tex_file.file_name().unwrap().to_string_lossy(), size);
                    success_count += 1;
                }
                Err(e) => {
                    panic!("Failed to convert {}: {}", tex_file.display(), e);
                }
            }
        }
        
        println!("Converted {}/{} files successfully", success_count, tex_files.len());
        assert_eq!(success_count, tex_files.len(), "Some files failed to convert");
    }
}
