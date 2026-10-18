//! Integration tests for InstanceManager and directory isolation.

use dv_launcher::instances::InstanceManager;
use tempfile::tempdir;

#[test]
fn test_instance_creation_and_isolation() {
    let temp = tempdir().unwrap();
    let mut inst_mgr = InstanceManager::new(temp.path()).unwrap();

    let inst = inst_mgr
        .create_instance("Test Cyber World", "1.20.4", "fabric", Some("0.15.7".to_string()))
        .unwrap();

    assert_eq!(inst.name, "Test Cyber World");
    assert_eq!(inst.minecraft_version, "1.20.4");
    assert_eq!(inst.mod_loader, "fabric");
    assert!(inst.instance_dir(temp.path()).exists());
    assert!(inst.game_dir(temp.path()).exists());
    assert!(inst.instance_dir(temp.path()).join("instance.json").exists());

    // Duplicate instance
    let cloned = inst_mgr.duplicate_instance(&inst.id, "Cloned World").unwrap();
    assert_ne!(cloned.id, inst.id);
    assert_eq!(cloned.name, "Cloned World");
    assert_eq!(cloned.minecraft_version, "1.20.4");

    // Delete cloned
    assert!(inst_mgr.delete_instance(&cloned.id).unwrap());
    assert!(!cloned.instance_dir(temp.path()).exists());

    // Clean up original
    assert!(inst_mgr.delete_instance(&inst.id).unwrap());
    assert!(!inst.instance_dir(temp.path()).exists());
}
