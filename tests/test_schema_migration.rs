use agent_memory_rs::storage::Database;
use std::fs;

#[test]
fn test_migration_to_v2() {
    let db_path = "/tmp/test_migration_v2.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to create database");

    // Verify all tables exist
    let tables: Vec<String> = db.execute(|conn| {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
        let tables = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tables)
    }).unwrap();

    // Check new tables
    assert!(tables.contains(&"episodes".to_string()), "episodes table missing");
    assert!(tables.contains(&"procedures".to_string()), "procedures table missing");
    assert!(tables.contains(&"daily_synopsis".to_string()), "daily_synopsis table missing");
    
    // Check new vector tables
    assert!(tables.contains(&"vec_episodes".to_string()), "vec_episodes table missing");
    assert!(tables.contains(&"vec_procedures".to_string()), "vec_procedures table missing");
    assert!(tables.contains(&"vec_synopsis".to_string()), "vec_synopsis table missing");

    // Verify schema version
    let version: i32 = db.execute(|conn| {
        Ok(conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))?)
    }).unwrap();
    assert_eq!(version, 1, "Schema version should be 1");

    // Verify episodes table structure
    let episode_columns: Vec<String> = db.execute(|conn| {
        let mut stmt = conn.prepare("PRAGMA table_info(episodes)")?;
        let columns = stmt.query_map([], |row| row.get(1))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(columns)
    }).unwrap();
    
    assert!(episode_columns.contains(&"event_type".to_string()));
    assert!(episode_columns.contains(&"context".to_string()));
    assert!(episode_columns.contains(&"valence".to_string()));
    assert!(episode_columns.contains(&"archived".to_string()));

    println!("✓ Migration to v2 successful");
    println!("✓ All tables created: {:?}", tables);
}
