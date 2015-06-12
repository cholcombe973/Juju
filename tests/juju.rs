extern crate juju;
use std::collections::HashMap;

#[test]
fn test_relation_ids(){

}

#[test]
fn test_relation_list(){

}

#[test]
fn test_leadership_election(){
    //create a context
    //pass into is_leader
    //check if it selects the proper leader
    let mut has_bigger_relations = HashMap::new();

    let mut has_lower_relations = HashMap::new();

    let should_be_leader = juju::Context{
        relation_type: "server".to_string(),
        relation_id: 1,
        unit: "gluster/1".to_string(),
        relations: has_bigger_relations,
    };
    let should_not_be_leader = juju::Context{
        relation_type: "server".to_string(),
        relation_id: 2,
        unit: "gluster/2".to_string(),
        relations: has_lower_relations,
    };
    assert!(juju::is_leader(&should_be_leader));
    assert_eq!(false, juju::is_leader(&should_not_be_leader));
}
