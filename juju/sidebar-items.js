initSidebarItems({"enum":[["JujuError",""],["LogLevel","An enum representing the available verbosity levels of the logging framework"],["StatusType","For information about what these StatusType variants mean see: Status reference"],["Transport",""]],"fn":[["action_fail","See Juju Actions for more information # Failures Returns stderr if the action_fail command fails"],["action_get","action_get gets the value of the parameter at the given key See Juju Actions for more information # Failures Returns stderr if the action_get command fails"],["action_get_all","action_get_all gets all values that are set See Juju Actions for more information # Failures Returns stderr if the action_get command fails"],["action_name","Get the name of the currently executing action # Failures Returns JujuError if the environment variable JUJU_ACTION_NAME does not exist"],["action_set","action_set permits the Action to set results in a map to be returned at completion of the Action. See Juju Actions for more information # Failures Returns stderr if the action_set command fails"],["action_tag","Get the tag of the currently executing action # Failures Returns JujuError if the environment variable JUJU_ACTION_TAG does not exist"],["action_uuid","Get the uuid of the currently executing action # Failures Returns JujuError if the environment variable JUJU_ACTION_UUID does not exist"],["add_metric","Add metric values See Juju Metrics for more information May only be called from the collect-metrics hook # Failures Returns stderr if the add_metric command fails"],["application_version_set","Charm authors may trigger this command from any hook to output what version of the application is running. This could be a package version, for instance postgres version 9.5. It could also be a build number or version control revision identifier, for instance git sha 6fb7ba68. # Failures Returns stderr if the action_get command fails"],["az_info","Get the availability zone # Failures Returns stderr if the meter_status command fails"],["close_port","This will hide a port on the unit.  The transport argument will indicate whether tcp or udp should be exposed # Failures Will return a String of the stderr if the call fails"],["config_get","This will return a configuration item that corresponds to the key passed in # Failures Will return a String of the stderr if the call fails"],["config_get_all","config_get_all will return all configuration options as a HashMap<String,String> # Failures Returns a String of if the configuration options are not able to be transformed into a HashMap"],["is_leader","Returns true/false if this unit is the leader # Failures Will return stderr as a String if the function fails to run # Examples ``` extern crate juju; let leader = match juju::is_leader(){   Ok(l) => l,   Err(e) => {     println!(\"Failed to run.  Error was {:?}\", e);     //Bail     return;   }, }; if leader{   println!(\"I am the leader!\"); }else{   println!(\"I am not the leader.  Maybe later I will be promoted\"); } ```"],["leader_get","Juju leader get value(s) # Failures Will return stderr as a String if the function fails to run"],["leader_set","Juju leader set value(s) # Failures Will return stderr as a String if the function fails to run"],["log","Log a message, at an optional log::LogLevel, to the Juju log"],["meter_info","Get the meter status information, if running in the meter-status-changed hook # Failures Returns stderr if the meter_info command fails"],["meter_status","Get the meter status, if running in the meter-status-changed hook # Failures Returns stderr if the meter_status command fails"],["open_port","This will expose a port on the unit.  The transport argument will indicate whether tcp or udp should be exposed # Failures Will return a String of the stderr if the call fails"],["process_hooks","Call this to process your cmd line arguments and call any needed hooks # Examples ```     extern crate juju;     extern crate log;"],["reboot","This will reboot your juju instance.  Examples of using this are when a new kernel is installed and the virtual machine or server needs to be rebooted to use it. # Failures Returns stderr if the reboot command fails"],["relation_get","Get relation information for the current unit # Failures Will return a String of the stderr if the call fails"],["relation_get_by_id","Get relation information using a specific relation ID. Used outside of relation hooks # Failures Will return a String of the stderr if the call fails"],["relation_get_by_unit","Get relation information for a specific unit # Failures Will return a String of the stderr if the call fails"],["relation_ids",""],["relation_ids_by_identifier","Gets the relation IDs by their identifier # Failures Will return a String of the stderr if the call fails"],["relation_list","Returns a list of all related units # Failures Will return a String of the stderr if the call fails"],["relation_list_by_id","Returns a list of all related units for the supplied identifier # Failures Will return a String of the stderr if the call fails"],["relation_set","Set relation information for the current unit # Failures Will return a String of the stderr if the call fails"],["relation_set_by_id","Sets relation information using a specific relation ID. Used outside of relation hooks # Failures Will return a String of the stderr if the call fails"],["status_get","Retrieve the previously set juju workload state # Failures Will return a String of the stderr if the call fails"],["status_set","Set the status of your unit to indicate to the Juju if everything is ok or something is wrong. See the Status enum for information about what can be set. # Failures Will return a String of the stderr if the call fails"],["storage_get","Return the location of the mounted storage device.  The mounted storage devices can be gotten by calling storage_list() and then passed into this function to get their mount location. # Failures Will return a String of the stderr if the call fails"],["storage_get_location","If storage drives were allocated to your unit this will get the path of them. In the storage-attaching hook this will tell you the location where the storage is attached to.  IE: /dev/xvdf for block devices or /mnt/{name} for filesystem devices # Failures Will return a String of the stderr if the call fails"],["storage_list","Used to list storage instances that are attached to the unit. The names returned may be passed through to storage_get # Failures Will return a String of the stderr if the call fails"],["unit_get_private_addr","This will return the private IP address associated with the unit. It can be very useful for services that require communicating with the other units related to it. # Failures Will return a String of the stderr if the call fails"],["unit_get_public_addr","This will return the public IP address associated with the unit. # Failures Will return a String of the stderr if the call fails"]],"macro":[["hook",""],["log",""],["status_set",""]],"mod":[["unitdata",""]],"struct":[["Config","A HashMap representation of the charm's config.yaml, with some extra features: - See which values in the HashMap have changed since the previous hook. - For values that have changed, see what the previous value was. - Store arbitrary data for use in a later hook."],["Context",""],["Hook",""],["Relation",""],["Status",""]]});