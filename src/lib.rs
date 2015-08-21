//! A library to interface with Juju.  For more information about Juju see
//! [Juju](https://jujucharms.com/docs/stable/about-juju)
//!
//! A hello world Juju charm example in Rust:
//! You will need a working Juju environment for this to function properly.  See [Setting up Juju]
//! (https://jujucharms.com/docs/stable/getting-started).  After Juju is functioning see
//! [What makes a Charm](https://jujucharms.com/docs/stable/authors-charm-components) for the base
//! components of a charm.
//!
//! Our src/main.rs will contain the following:
//! # Examples
//! ```
//! extern crate juju;
//! use std::env;
//!
//! fn config_changed()->Result<(), String>{
//!     juju::log(&"Hello Juju from Rust!".to_string());
//!     return Ok(());
//! }
//!
//! fn main(){
//!    let args: Vec<String> = env::args().collect();
//!    if args.len() > 0{
//!         let mut hook_registry: Vec<juju::Hook> = Vec::new();
//!
//!         //Register our hooks with the Juju library
//!         hook_registry.push(juju::Hook{
//!             name: "config-changed".to_string(),
//!             callback: Box::new(config_changed),
//!         });
//!         let result =  juju::process_hooks(args, hook_registry);
//!
//!         if result.is_err(){
//!             juju::log(&format!("Hook failed with error: {:?}", result.err()));
//!         }else{
//!             juju::log(&"Hook call was successful!".to_string());
//!         }
//!    }
//! }
//! ```
//! Now you can build with `cargo build ` and install the binary in the hooks directory.
//!
//! Create a symlink in the hooks directory with `ln -s hello-world config-changed`.  Juju will
//! attempt to run that symlink and our Juju library will map that to our config_changed function.
//!
//! We can test our hello-world charm by deploying with juju and watching the debug logs. See
//! [Deploying a Charm](https://jujucharms.com/docs/stable/charms-deploying) for more information.
//!
//! You should see a message in juju debug-log like this `unit-hello-world-0[6229]: 2015-08-21 16:16:05 INFO unit.hello-world/0.juju-log server.go:254 Hello Juju from Rust!`
//!

use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub enum Transport {
    Tcp,
    Udp,
}

impl Transport {
    /// Returns a String representation of the enum variant
    fn to_string(self) -> String {
        match self {
            Transport::Tcp => "tcp".to_string(),
            Transport::Udp => "udp".to_string(),
        }
    }
}

#[derive(Debug)]
/// For information about what these StatusType variants mean see: [Status reference]
/// (https://jujucharms.com/docs/stable/reference-status)
pub enum StatusType{
    Maintenance,
    Waiting,
    Active,
    Blocked
}

impl StatusType {
    /// Returns a String representation of the enum variant
    pub fn to_string(self) -> String {
        match self {
            StatusType::Maintenance => "maintenance".to_string(),
            StatusType::Waiting => "waiting".to_string(),
            StatusType::Active => "active".to_string(),
            StatusType::Blocked => "blocked".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Status{
    /// The type of status
    pub status_type: StatusType,
    /// A message to show alongside the status
    pub message: String,
}

#[derive(Debug)]
pub struct Context{
    /// The scope for the current relation hook
    pub relation_type: String,
    /// The relation ID for the current relation hook
    pub relation_id: usize,
    /// Local unit ID
    pub unit: String,
    /// relation data for all related units
    pub relations: HashMap<String,String>,
}

impl Context{
    ///Constructs a new `Context`
    ///Creates a context that's filled out from the env variables
    /// # Example usage
    /// ```
    /// extern crate juju;
    /// let context = juju::Context::new_from_env();
    /// ```

    pub fn new_from_env() -> Context{
        let relations: HashMap<String,String> = HashMap::new();

        //This variable is useless.  It only shows "server" for everything
        let relation_type = env::var("JUJU_RELATION").unwrap_or("".to_string());
        let relation_id_str = env::var("JUJU_RELATION_ID").unwrap_or("".to_string());
        let parts: Vec<&str> = relation_id_str.split(":").collect();
        let relation_id: usize = parts[1].parse::<usize>().unwrap();
        let unit = env::var("JUJU_UNIT_NAME").unwrap_or("".to_string());

        Context{
            relation_type: relation_type,
            relation_id: relation_id,
            unit: unit,
            relations: relations,
        }
    }
}

#[derive(Debug)]
pub struct Relation {
    /// The name of a unit related to your service
    pub name: String,
    /// The id of the unit related to your service
    pub id: usize
}

pub struct Hook {
    /// The name of the hook to call
    pub name: String,
    /// A function to call when Juju calls this hook
    /// # Failures
    /// Your function passed in needs to return a String on error so that users will
    /// know what happened.  Ideally this should also be logged with juju::log
    pub callback: Box<Fn()->Result<(),String>>,
}

/// Returns 0 if the process completed successfully.
/// #Failures
/// Returns a String of the stderr if the process failed to execute
fn process_output(output: std::process::Output)->Result<i32, String>{
    let status = output.status;

    if status.success(){
        return Ok(0);
    }else{
        return Err(
            try!(String::from_utf8(output.stderr).map_err(|e| e.to_string()))
        );
    }
}

/// Logs the msg passed to it
/// # Examples
/// ```
/// extern crate juju;
/// let error = "Error information";
/// juju::log(&format!("Super important info. Error {}", error));
/// ```
/// # Failures
/// Does not return anything on failure.  Java has the same semantics.  I'm still wondering
/// if this is the right thing to do.
pub fn log(msg: &String){
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(msg.clone());

    //Ignoring errors if they happen.
    //TODO: should this return success/failure?  It makes the code ugly
    run_command("juju-log", &arg_list, false).is_ok();
}

/// This will reboot your juju instance.  Examples of using this are when a new kernel is installed
/// and the virtual machine or server needs to be rebooted to use it.
/// # Failures
/// Returns stderr if the reboot command fails
pub fn reboot()->Result<i32,String>{
    let output = try!(run_command_no_args("juju-reboot", true).map_err(|e| e.to_string()));
    return process_output(output);
}
/// This will return the private IP address associated with the unit.
/// It can be very useful for services that require communicating with the other units related
/// to it.
pub fn unit_get_private_addr() ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("private-address".to_string());
    let output = try!(run_command("unit-get", &arg_list, false).map_err(|e| e.to_string()));
    let private_addr: String = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(private_addr.trim().to_string());
}

/// This will return the public IP address associated with the unit.
pub fn unit_get_public_addr() ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("public-address".to_string());
    let output = try!(run_command("unit-get", &arg_list, false).map_err(|e| e.to_string()));
    let public_addr = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(public_addr.trim().to_string());
}

/// This will return a configuration item that corresponds to the key passed in
pub fn config_get(key: &String) ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(key.clone());
    let output = try!(run_command("config-get", &arg_list, false).map_err(|e| e.to_string()));
    let value = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(value.trim().to_string());
}

/// config_get_all will return all configuration options as a HashMap<String,String>
/// # Failures
/// Returns a String of if the configuration options are not able to be transformed into a HashMap
pub fn config_get_all() -> Result<HashMap<String,String>, String>{
    let mut values: HashMap<String,String> = HashMap::new();

    let arg_list: Vec<String>  = vec!["--all".to_string()];
    let output = try!(run_command("config-get", &arg_list, false).map_err(|e| e.to_string()));
    let output_str = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    /*  Example output:
        "brick_paths: /mnt/brick1 /mnt/brick2\ncluster_type: Replicate\n"
    */
    //For each line split at : and load the parts into the HashMap
    for line in output_str.lines(){
        let parts: Vec<&str> = line.split(":").filter(|s| !s.is_empty()).collect::<Vec<&str>>();
        if ! parts.len() == 2{
            //Skipping this possibly bogus value
           continue;
        }
        let key = try!(parts.get(0).ok_or(
            format!("Unable to get key from config-get from parts: {:?}", parts)));
        let value = try!(parts.get(1).ok_or(
            format!("Unable to get value from config-get from parts: {:?}", parts)));
        values.insert(key.to_string(), value.to_string());
    }

    return Ok(values);
}

/// This will expose a port on the unit.  The transport argument will indicate whether tcp or udp
/// should be exposed
pub fn open_port(port: usize, transport: Transport)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string(), transport.to_string());

    arg_list.push(port_string);
    let output = try!(run_command("open-port", &arg_list, false).map_err(|e| e.to_string()));
    return process_output(output);
}

/// This will hide a port on the unit.  The transport argument will indicate whether tcp or udp
/// should be exposed
pub fn close_port(port: usize, transport: Transport)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string() , transport.to_string());

    arg_list.push(port_string);
    let output = try!(run_command("close-port", &arg_list, false).map_err(|e| e.to_string()));
    return process_output(output);
}

pub fn relation_set(key: &str, value: &str)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let arg = format!("{}={}", key.clone(), value);

    arg_list.push(arg);
    let output = try!(run_command("relation-set", &arg_list, false).map_err(|e| e.to_string()));
    return process_output(output);
}

pub fn relation_get(key: &String) -> Result<String,String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(key.clone());
    let output = try!(run_command("relation-get", &arg_list, false).map_err(|e| e.to_string()));
    let value = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(value);
}

pub fn relation_get_by_unit(key: &String, unit: &Relation) -> Result<String,String>{
    let mut arg_list: Vec<String>  = Vec::new();
    //arg_list.push("-r".to_string());
    arg_list.push(key.clone());
    arg_list.push(format!("{}/{}", unit.name , unit.id.to_string()));
    //let output = run_command("relation-get", &arg_list, false);
    //return String::from_utf8(output.stdout).unwrap();
    let output = try!(run_command("relation-get", &arg_list, false).map_err(|e| e.to_string()));
    let relation = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(relation);
    //return try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
}

/// Returns a list of all related units
/// # Failures
/// Will return a String of the stderr if the call fails

pub fn relation_list() ->Result<Vec<Relation>, String>{
    let mut related_units: Vec<Relation> = Vec::new();

    let output = try!(run_command_no_args("relation-list", false).map_err(|e| e.to_string()));
    let output_str =  try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));

    log(&format!("relation-list output: {}", output_str));

    for line in output_str.lines(){
        let v: Vec<&str> = line.split('/').collect();
        let id: usize = try!(v[1].parse::<usize>().map_err(|e| e.to_string()));
        let r: Relation = Relation{
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

pub fn relation_ids() ->Result<Vec<Relation>, String>{
    let mut related_units: Vec<Relation> = Vec::new();
    let output = try!(run_command_no_args("relation-ids", false).map_err(|e| e.to_string()));
    let output_str: String =  try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    log(&format!("relation-ids output: {}", output_str));

    for line in output_str.lines(){
        let v: Vec<&str> = line.split(':').collect();
        let id: usize = try!(v[1].parse::<usize>().map_err(|e| e.to_string()));
        let r: Relation = Relation{
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

/// Set the status of your unit to indicate to the Juju if everything is ok or something is wrong.
/// See the Status enum for information about what can be set.
pub fn status_set(status: Status)->Result<i32,String>{
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(status.status_type.to_string());
    arg_list.push(status.message);

    let output = try!(run_command("status-set", &arg_list, false).map_err(|e| e.to_string()));
    return process_output(output);
}

/// If storage drives were allocated to your unit this will get the path of them.
pub fn storage_get_location() ->Result<String, String>{
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push("location".to_string());
    let output = try!(run_command("storage-get", &arg_list, false).map_err(|e| e.to_string()));
    return Ok(try!(String::from_utf8(output.stdout).map_err(|e| e.to_string())));
}

/// Call this to process your cmd line arguments and call any needed hooks
/// # Examples
/// ```
///     extern crate juju;
///     use std::env;
///
///     fn config_changed()->Result<(), String>{
///         //Do nothing
///         return Ok(());
///    }
///
///    let args: Vec<String> = env::args().collect();
///    if args.len() > 0{
///         let mut hook_registry: Vec<juju::Hook> = Vec::new();
///
///         //Register our hooks with the Juju library
///         hook_registry.push(juju::Hook{
///             name: "config-changed".to_string(),
///             callback: Box::new(config_changed),
///         });
///         let result =  juju::process_hooks(args, hook_registry);

///         if result.is_err(){
///             juju::log(&format!("Hook failed with error: {:?}", result.err()));
///         }
///    }
/// ```
///
pub fn process_hooks(args: Vec<String>, registry: Vec<Hook>)->Result<(),String>{
    let path = std::path::Path::new(args[0].trim());

    let filename = match path.file_name(){
        Some(filename) => filename,
        None => {
            return Err(format!("Unable to parse filename from {:?}", path));
        },
    };

    let match_str = match filename.to_str(){
        Some(filename) => filename,
        None => {
            return Err(format!("Failed to transform filename into string {:?}.  Bad symlink name perhaps? Bailing", filename));
        },
    };

    for hook in registry {
        if hook.name == match_str{
            return (*hook.callback)();
        }
    }
    return Err(format!("Warning: Unknown callback for hook {}", match_str));
}

/// Returns true/false if this unit is the leader
/// # Failures
/// Will return stderr as a String if the function fails to run
/// # Examples
/// ```
/// extern crate juju;
/// let leader = match juju::is_leader(){
///   Ok(l) => l,
///   Err(e) => {
///     println!("Failed to run.  Error was {}", e);
///     //Bail
///     return;
///   },
/// };
/// if leader{
///   println!("I am the leader!");
/// }else{
///   println!("I am not the leader.  Maybe later I will be promoted");
/// }
/// ```
///
pub fn is_leader()->Result<bool, String>{
    let output = try!(run_command_no_args("is-leader", false).map_err(|e| e.to_string()));
    let output_str: String =  try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    match output_str.trim().as_ref() {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Ok(false),
    }
}

fn run_command_no_args(command: &str, as_root: bool)-> Result<std::process::Output, String>{
    if as_root{
        let mut cmd = std::process::Command::new("sudo");
        //println!("Running command: {:?}", cmd);
        let output = try!(cmd.output().map_err(|e| e.to_string()));
        return Ok(output);
    }else{
       let mut cmd = std::process::Command::new(command);
        //println!("Running command: {:?}", cmd);
        let output = try!(cmd.output().map_err(|e| e.to_string()));
        return Ok(output);
    }
}

fn run_command(command: &str, arg_list: &Vec<String>, as_root: bool) -> Result<std::process::Output, String>{
    if as_root{
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg(command);
        for arg in arg_list{
            cmd.arg(&arg);
        }
        //println!("Running command: {:?}", cmd);
        let output = try!(cmd.output().map_err(|e| e.to_string()));
        return Ok(output);
    }else{
       let mut cmd = std::process::Command::new(command);
        for arg in arg_list{
            cmd.arg(&arg);
        }
        //println!("Running command: {:?}", cmd);
        let output = try!(cmd.output().map_err(|e| e.to_string()));
        return Ok(output);
    }
}
