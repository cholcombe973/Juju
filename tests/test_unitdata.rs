extern crate juju;

use std::collections::HashMap;
use std::path::PathBuf;

use juju::JujuError;
use juju::unitdata::*;

#[test]
fn get_set_delete_keys() {
    let unitdata = Storage::new(Some(PathBuf::from("/tmp/unitdata.db")))
        .expect("Failed to connect to database");
    unitdata.set("foo", "bar").unwrap();
    let value: Result<Option<String>, JujuError> = unitdata.get("foo");
    assert_eq!("bar", value.unwrap().unwrap());

    let mut updates: HashMap<String, String> = HashMap::new();
    updates.insert("foo".to_string(), "baz".to_string());
    unitdata.update(updates, None).expect("Updates failed");

    let value: Result<Option<String>, JujuError> = unitdata.get("foo");
    assert_eq!("baz", value.unwrap().unwrap());

    // Test that delete works
    unitdata.unset("foo").unwrap();

    // Set a few more keys so we can unset them all at once
    unitdata.set("foo", "bar").unwrap();
    unitdata.set("foo2", "bar").unwrap();
    unitdata.set("foo3", "bar").unwrap();

    let results = unitdata.getrange("foo", false).unwrap();
    // check that we got everything
    assert_eq!(results.get("foo").unwrap(), "bar");
    assert_eq!(results.get("foo2").unwrap(), "bar");
    assert_eq!(results.get("foo3").unwrap(), "bar");;

    // This time get the same values but strip the prefix off of the keys
    let results_2 = unitdata.getrange("foo", true).unwrap();
    assert_eq!(results_2.get("").unwrap(), "bar");
    assert_eq!(results_2.get("2").unwrap(), "bar");
    assert_eq!(results_2.get("3").unwrap(), "bar");;

    let rows_deleted =
        unitdata.unsetrange(Some(vec!["foo".to_string(), "foo2".to_string(), "foo3".to_string()]),
                        None)
            .unwrap();
    assert_eq!(rows_deleted, 3);
}
