use zotero_rdf::{Extractor, parse_file};

#[test]
fn test_attachment_path_extraction() {
    // Parse the RDF file with attachments
    let graph = parse_file("rdfs/simulation-with-attachments/simulation-with-attachments.rdf")
        .expect("Failed to parse RDF file");

    // Extract items
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // Find items with attachments
    let items_with_attachments: Vec<_> = items
        .iter()
        .filter(|item| !item.attachments.is_empty())
        .collect();

    println!(
        "Found {} items with attachments",
        items_with_attachments.len()
    );

    // Verify that attachments have path fields
    for item in &items_with_attachments {
        println!("\nItem: {}", item.title.as_deref().unwrap_or("(untitled)"));
        println!("  Attachments count: {}", item.attachments.len());

        for attachment in &item.attachments {
            println!(
                "    - Title: {}",
                attachment.title.as_deref().unwrap_or("(untitled)")
            );
            println!("      Path: {:?}", attachment.path);
            println!("      URL: {:?}", attachment.url);
            println!("      Content-Type: {:?}", attachment.content_type);

            // Verify path is extracted (should not be None for real attachments)
            assert!(
                attachment.path.is_some(),
                "Attachment '{}' should have a path field",
                attachment.title.as_deref().unwrap_or("(untitled)")
            );

            // Verify path starts with "files/" for local attachments
            let path = attachment.path.as_ref().unwrap();
            assert!(
                path.starts_with("files/"),
                "Path should start with 'files/', got: {}",
                path
            );
        }
    }

    // Verify at least one item has attachments with paths
    assert!(
        !items_with_attachments.is_empty(),
        "Should have at least one item with attachments"
    );
}

#[test]
fn test_specific_attachment_path() {
    // Parse the RDF file
    let graph = parse_file("rdfs/simulation-with-attachments/simulation-with-attachments.rdf")
        .expect("Failed to parse RDF file");

    // Extract items
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // Find the specific item we know has attachments
    let item = items
        .iter()
        .find(|item| {
            item.title
                .as_ref()
                .map(|t| t.contains("Laser Triangulation"))
                .unwrap_or(false)
        })
        .expect("Should find the laser triangulation item");

    println!("Found item: {:?}", item.title);
    println!("Attachments: {}", item.attachments.len());

    // Verify it has attachments
    assert!(!item.attachments.is_empty(), "Item should have attachments");

    // Find the PDF attachment
    let pdf_attachment = item
        .attachments
        .iter()
        .find(|a| a.content_type.as_deref() == Some("application/pdf"))
        .expect("Should have a PDF attachment");

    println!("PDF Attachment:");
    println!("  Title: {:?}", pdf_attachment.title);
    println!("  Path: {:?}", pdf_attachment.path);
    println!("  URL: {:?}", pdf_attachment.url);

    // Verify path is present and correct
    assert!(
        pdf_attachment.path.is_some(),
        "PDF attachment should have a path"
    );

    let path = pdf_attachment.path.as_ref().unwrap();
    assert!(
        path.contains("2474"),
        "Path should contain '2474', got: {}",
        path
    );
    assert!(
        path.ends_with(".pdf"),
        "Path should end with '.pdf', got: {}",
        path
    );

    // Verify URL is also present (from dc:identifier)
    assert!(
        pdf_attachment.url.is_some(),
        "PDF attachment should have a URL"
    );
    let url = pdf_attachment.url.as_ref().unwrap();
    assert!(
        url.starts_with("http"),
        "URL should start with 'http', got: {}",
        url
    );
}

#[test]
fn test_attachment_path_vs_url() {
    // This test verifies that path (local file) and url (original source) are different
    let graph = parse_file("rdfs/simulation-with-attachments/simulation-with-attachments.rdf")
        .expect("Failed to parse RDF file");

    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();

    // Find an item with both path and url
    for item in items.iter() {
        for attachment in item.attachments.iter() {
            if let (Some(path), Some(url)) = (&attachment.path, &attachment.url) {
                println!("\nAttachment: {:?}", attachment.title);
                println!("  Path (local): {}", path);
                println!("  URL (source): {}", url);

                // Path should be a relative file path
                assert!(
                    !path.starts_with("http"),
                    "Path should not be an HTTP URL, got: {}",
                    path
                );

                // URL should be an HTTP URL
                assert!(
                    url.starts_with("http"),
                    "URL should be an HTTP URL, got: {}",
                    url
                );

                // They should be different
                assert_ne!(path, url, "Path and URL should be different values");
            }
        }
    }
}
