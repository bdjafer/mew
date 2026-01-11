//! Integration tests for level-1 ontologies

use mew_testgen::{TestConfig, TestGenerator, TestExecutor};
use mew_testgen::trust::TrustAuditor;
use mew_testgen::report::ReportGenerator;

const BOOKMARKS_ONTOLOGY: &str = r##"
ontology Bookmarks {
  node Bookmark {
    url: String [required],
    title: String [required],
    description: String?,
    is_favorite: Bool = false,
    visit_count: Int = 0
  }

  node Folder {
    name: String [required]
  }

  node Tag {
    name: String [required],
    color: String = "#808080"
  }

  edge in_folder(bookmark: Bookmark, folder: Folder)
  edge parent_folder(child: Folder, parent: Folder)
  edge tagged(bookmark: Bookmark, tag: Tag)
}
"##;

const LIBRARY_ONTOLOGY: &str = r#"
ontology Library {
  node Book {
    title: String [required],
    isbn: String [required, unique],
    pages: Int [>= 1],
    is_available: Bool = true
  }

  node Author {
    name: String [required]
  }

  node Member {
    name: String [required],
    email: String [required, unique]
  }

  edge wrote(author: Author, book: Book)
  edge borrowed(member: Member, book: Book)
}
"#;

const CONTACTS_ONTOLOGY: &str = r#"
ontology Contacts {
  node Person {
    name: String [required],
    email: String?,
    phone: String?
  }

  node Company {
    name: String [required],
    industry: String?
  }

  node Group {
    name: String [required]
  }

  edge works_at(person: Person, company: Company)
  edge member_of(person: Person, group: Group)
  edge knows(a: Person, b: Person)
}
"#;

#[test]
fn test_schema_analysis_bookmarks() {
    use mew_testgen::SchemaAnalyzer;

    let schema = SchemaAnalyzer::analyze(BOOKMARKS_ONTOLOGY).unwrap();

    // Check node types
    assert!(schema.node_types.contains_key("Bookmark"));
    assert!(schema.node_types.contains_key("Folder"));
    assert!(schema.node_types.contains_key("Tag"));

    // Check Bookmark attributes
    let bookmark = &schema.node_types["Bookmark"];
    assert_eq!(bookmark.attrs.len(), 5);

    let url_attr = bookmark.attrs.iter().find(|a| a.name == "url").unwrap();
    assert!(url_attr.required);
    assert!(!url_attr.nullable);

    let desc_attr = bookmark.attrs.iter().find(|a| a.name == "description").unwrap();
    assert!(desc_attr.nullable);

    // Check edge types
    assert!(schema.edge_types.contains_key("in_folder"));
    assert!(schema.edge_types.contains_key("tagged"));
}

#[test]
fn test_schema_analysis_library() {
    use mew_testgen::SchemaAnalyzer;

    let schema = SchemaAnalyzer::analyze(LIBRARY_ONTOLOGY).unwrap();

    // Check node types
    assert!(schema.node_types.contains_key("Book"));
    assert!(schema.node_types.contains_key("Author"));
    assert!(schema.node_types.contains_key("Member"));

    // Check Book attributes
    let book = &schema.node_types["Book"];
    let isbn_attr = book.attrs.iter().find(|a| a.name == "isbn").unwrap();
    assert!(isbn_attr.required);
    assert!(isbn_attr.unique);

    let pages_attr = book.attrs.iter().find(|a| a.name == "pages").unwrap();
    assert!(pages_attr.min.is_some());
}

#[test]
fn test_schema_analysis_contacts() {
    use mew_testgen::SchemaAnalyzer;

    let schema = SchemaAnalyzer::analyze(CONTACTS_ONTOLOGY).unwrap();

    // Check node types
    assert!(schema.node_types.contains_key("Person"));
    assert!(schema.node_types.contains_key("Company"));
    assert!(schema.node_types.contains_key("Group"));

    // Check edge types
    assert!(schema.edge_types.contains_key("knows"));
    let knows = &schema.edge_types["knows"];
    assert_eq!(knows.params.len(), 2);
}

#[test]
fn test_world_generation_bookmarks() {
    use mew_testgen::{SchemaAnalyzer, WorldGenerator};
    use rand::SeedableRng;

    let schema = SchemaAnalyzer::analyze(BOOKMARKS_ONTOLOGY).unwrap();
    let config = TestConfig::default();
    let mut gen = WorldGenerator::new(&schema, config.clone());
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let world = gen.generate(&mut rng).unwrap();

    // Should have nodes for each type
    assert_eq!(world.nodes_by_type.get("Bookmark").map(|v| v.len()).unwrap_or(0), config.nodes_per_type);
    assert_eq!(world.nodes_by_type.get("Folder").map(|v| v.len()).unwrap_or(0), config.nodes_per_type);
    assert_eq!(world.nodes_by_type.get("Tag").map(|v| v.len()).unwrap_or(0), config.nodes_per_type);

    // Should have some edges
    assert!(!world.edges.is_empty());
}

#[test]
fn test_query_generation_bookmarks() {
    use mew_testgen::{SchemaAnalyzer, WorldGenerator, QueryGenerator};
    use rand::SeedableRng;

    let schema = SchemaAnalyzer::analyze(BOOKMARKS_ONTOLOGY).unwrap();
    let config = TestConfig::default();
    let mut gen = WorldGenerator::new(&schema, config.clone());
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let world = gen.generate(&mut rng).unwrap();
    let mut query_gen = QueryGenerator::new(&schema, &world);
    let queries = query_gen.generate(10, &mut rng).unwrap();

    assert_eq!(queries.len(), 10);
    for q in &queries {
        assert!(q.statement.starts_with("MATCH"));
    }
}

#[test]
fn test_full_suite_generation_bookmarks() {
    let config = TestConfig::default()
        .with_seed(42)
        .with_nodes_per_type(3)
        .with_query_count(5)
        .with_mutation_count(3);

    let mut generator = TestGenerator::new(config);
    let suite = generator.generate_suite(BOOKMARKS_ONTOLOGY).unwrap();

    // Should have test cases
    assert!(!suite.test_cases.is_empty());
    // May have fewer than 8 if some mutations couldn't be generated
    assert!(suite.test_cases.len() >= 5, "Expected at least 5 test cases, got {}", suite.test_cases.len());

    // Check that we have different types of tests
    let query_count = suite.test_cases.iter()
        .filter(|t| t.tags.contains(&"match".to_string()))
        .count();
    assert!(query_count > 0);

    let mutation_count = suite.test_cases.iter()
        .filter(|t| t.tags.contains(&"spawn".to_string()) || t.tags.contains(&"link".to_string()))
        .count();
    assert!(mutation_count > 0);
}

#[test]
fn test_full_suite_generation_all_level1() {
    for (name, source) in [
        ("Bookmarks", BOOKMARKS_ONTOLOGY),
        ("Library", LIBRARY_ONTOLOGY),
        ("Contacts", CONTACTS_ONTOLOGY),
    ] {
        let config = TestConfig::default()
            .with_seed(42)
            .with_nodes_per_type(3)
            .with_query_count(5)
            .with_mutation_count(3);

        let mut generator = TestGenerator::new(config);
        let result = generator.generate_suite(source);

        assert!(result.is_ok(), "Failed to generate suite for {}: {:?}", name, result.err());

        let suite = result.unwrap();
        assert!(!suite.test_cases.is_empty(), "No test cases for {}", name);

        println!("{}: {} test cases generated", name, suite.test_cases.len());
    }
}
