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
//! #[macro_use]
//! extern crate juju;
//! extern crate log;
//! use log::LogLevel;
//!
//! fn config_changed()->Result<(), String>{
//!     juju::log("Hello Juju from Rust!", Some(LogLevel::Debug));
//!     return Ok(());
//! }
//!
//! fn main(){
//!     let hook_registry: Vec<juju::Hook> = vec![
//!         hook!("config-changed", config_changed)
//!     ];
//!     let result =  juju::process_hooks(hook_registry);
//!
//!     if result.is_err(){
//!         juju::log(&format!("Hook failed with error: {:?}", result.err()),
//!             Some(LogLevel::Error));
//!     }else{
//!         juju::log("Hook call was successful!", Some(LogLevel::Debug));
//!     }
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
//! You should see a message in juju debug-log like this `unit-hello-world-0[6229]: 2015-08-21
//! 16:16:05 INFO unit.hello-world/0.juju-log server.go:254 Hello Juju from Rust!`
//!

extern crate charmhelpers;
extern crate log;
extern crate memchr;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

pub use log::LogLevel;
use memchr::memchr;

pub use charmhelpers::core::hookenv::log;

pub mod macros;
pub mod unitdata;

// Custom error handling for the library
#[derive(Debug)]
pub enum JujuError {
    AddrParseError(std::net::AddrParseError),
    FromUtf8Error(std::string::FromUtf8Error),
    IoError(io::Error),
    ParseIntError(std::num::ParseIntError),
    RusqliteError(rusqlite::Error),
    SerdeError(serde_json::Error),
    VarError(std::env::VarError),
}

impl JujuError {
    fn new(err: String) -> JujuError {
        JujuError::IoError(io::Error::new(std::io::ErrorKind::Other, err))
    }

    pub fn to_string(&self) -> String {
        match *self {
            JujuError::AddrParseError(ref err) => err.description().to_string(),
            JujuError::FromUtf8Error(ref err) => err.description().to_string(),
            JujuError::IoError(ref err) => err.description().to_string(),
            JujuError::ParseIntError(ref err) => err.description().to_string(),
            JujuError::RusqliteError(ref err) => err.description().to_string(),
            JujuError::SerdeError(ref err) => err.description().to_string(),
            JujuError::VarError(ref err) => err.description().to_string(),
        }
    }
}
impl fmt::Display for JujuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Error for JujuError {
    fn description(&self) -> &str {
        match *self {
            JujuError::AddrParseError(ref err) => err.description(),
            JujuError::FromUtf8Error(ref err) => err.description(),
            JujuError::IoError(ref err) => err.description(),
            JujuError::ParseIntError(ref err) => err.description(),
            JujuError::RusqliteError(ref err) => err.description(),
            JujuError::SerdeError(ref err) => err.description(),
            JujuError::VarError(ref err) => err.description(),
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            JujuError::AddrParseError(ref err) => err.cause(),
            JujuError::FromUtf8Error(ref err) => err.cause(),
            JujuError::IoError(ref err) => err.cause(),
            JujuError::ParseIntError(ref err) => err.cause(),
            JujuError::SerdeError(ref err) => err.cause(),
            JujuError::RusqliteError(ref err) => err.cause(),
            JujuError::VarError(ref err) => err.cause(),
        }
    }
}


impl From<io::Error> for JujuError {
    fn from(err: io::Error) -> JujuError {
        JujuError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for JujuError {
    fn from(err: std::string::FromUtf8Error) -> JujuError {
        JujuError::FromUtf8Error(err)
    }
}

impl From<std::num::ParseIntError> for JujuError {
    fn from(err: std::num::ParseIntError) -> JujuError {
        JujuError::ParseIntError(err)
    }
}

impl From<std::env::VarError> for JujuError {
    fn from(err: std::env::VarError) -> JujuError {
        JujuError::VarError(err)
    }
}

impl From<rusqlite::Error> for JujuError {
    fn from(err: rusqlite::Error) -> JujuError {
        JujuError::RusqliteError(err)
    }
}

impl From<serde_json::Error> for JujuError {
    fn from(err: serde_json::Error) -> JujuError {
        JujuError::SerdeError(err)
    }
}

impl From<std::net::AddrParseError> for JujuError {
    fn from(err: std::net::AddrParseError) -> JujuError {
        JujuError::AddrParseError(err)
    }
}


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
pub enum StatusType {
    Maintenance,
    Waiting,
    Active,
    Blocked,
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
pub struct Status {
    /// The type of status
    pub status_type: StatusType,
    /// A message to show alongside the status
    pub message: String,
}

#[derive(Debug)]
pub struct Context {
    /// The scope for the current relation hook
    pub relation_type: String,
    /// The relation ID for the current relation hook
    pub relation_id: usize,
    /// Local unit ID
    pub unit: String,
    /// relation data for all related units
    pub relations: HashMap<String, String>,
}

impl Context {
    /// Constructs a new `Context`
    /// Creates a context that's filled out from the env variables
    /// # Example usage
    /// ```
    /// extern crate juju;
    /// let context = juju::Context::new_from_env();
    /// ```

    pub fn new_from_env() -> Context {
        let relations: HashMap<String, String> = HashMap::new();

        // This variable is useless.  It only shows "server" for everything
        let relation_type = env::var("JUJU_RELATION").unwrap_or("".to_string());
        let relation_id_str = env::var("JUJU_RELATION_ID").unwrap_or("".to_string());
        let parts: Vec<&str> = relation_id_str.split(":").collect();
        let relation_id: usize;
        if parts.len() > 1 {
            relation_id = parts[1].parse::<usize>().unwrap_or(0);
        } else {
            relation_id = 0;
        }
        let unit = env::var("JUJU_UNIT_NAME").unwrap_or("".to_string());

        Context {
            relation_type: relation_type,
            relation_id: relation_id,
            unit: unit,
            relations: relations,
        }
    }
}

/// A HashMap representation of the charm's config.yaml, with some
/// extra features:
/// - See which values in the HashMap have changed since the previous hook.
/// - For values that have changed, see what the previous value was.
/// - Store arbitrary data for use in a later hook.
#[derive(Debug)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    /// Create a new Config and automatically load the previous config values if the
    /// .juju-persistent-config is present
    /// The .juju-persistent-config is also saved automatically when this struct is dropped.
    pub fn new() -> Result<Self, JujuError> {
        if Path::new(".juju-persistent-config").exists() {
            // Load the values from disk
            let mut file = OpenOptions::new().read(true).open(".juju-persistent-config")?;
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let previous_values: HashMap<String, String> = serde_json::from_str(&s)?;
            Ok(Config { values: previous_values })
        } else {
            // Initalize with all current values
            let current_values = config_get_all()?;
            Ok(Config { values: current_values })
        }
    }

    /// Return the current value for this key
    pub fn get(self, key: &str) -> Result<Option<String>, JujuError> {
        let current_value = config_get(key)?;
        Ok(current_value)
    }

    /// Return true if the current value for this key is different from
    /// the previous value.
    pub fn changed(self, key: &str) -> Result<bool, JujuError> {
        match self.values.get(key) {
            Some(previous_value) => {
                let current_value = config_get(key)?;
                match current_value {
                    Some(value) => Ok(&value != previous_value),
                    None => Ok(true),
                }
                //Ok(&current_value != previous_value)
            }
            // No previous key
            None => Ok(true),
        }
    }

    /// Return previous value for this key, or None if there
    /// is no previous value.
    pub fn previous(self, key: &str) -> Option<String> {
        match self.values.get(key) {
            Some(previous_value) => Some(previous_value.clone()),
            None => None,
        }
    }
}

impl Drop for Config {
    // Automatic saving of the .juju-persistent-config file when this struct is dropped
    fn drop(&mut self) {
        let mut file = match OpenOptions::new()
                  .write(true)
                  .truncate(true)
                  .create(true)
                  .open(".juju-persistent-config") {
            Ok(f) => f,
            Err(e) => {
                log(&format!("Unable to open .juju-persistent-config file for writing. Err: {}",
                             e),
                    Some(LogLevel::Error));
                return;
            }
        };
        let serialized = match serde_json::to_string(&self.values) {
            Ok(f) => f,
            Err(e) => {
                log(&format!("Unable to serialize Config values: {:?}.  Err: {}",
                             &self.values,
                             e),
                    Some(LogLevel::Error));
                return;
            }
        };
        match file.write(&serialized.as_bytes()) {
            Ok(bytes_written) => {
                log(&format!(".juju-persistent-config saved.  Wrote {} bytes",
                             bytes_written),
                    Some(LogLevel::Debug));
            }
            Err(e) => {
                log(&format!("Unable to write to .juju-persistent-config Err: {}", e),
                    Some(LogLevel::Error));
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct Relation {
    /// The name of a unit related to your service
    pub name: String,
    /// The id of the unit related to your service
    pub id: usize,
}

#[derive(Debug,PartialEq)]
pub struct Hook {
    /// The name of the hook to call
    pub name: String,
    /// A function to call when Juju calls this hook
    /// # Failures
    /// Your function passed in needs to return a String on error so that users will
    /// know what happened.  Ideally this should also be logged with juju::log
    pub callback: fn() -> Result<(), String>,
}

/// Returns 0 if the process completed successfully.
/// #Failures
/// Returns a String of the stderr if the process failed to execute
fn process_output(output: std::process::Output) -> Result<i32, JujuError> {
    let status = output.status;

    if status.success() {
        return Ok(0);
    } else {
        return Err(JujuError::new(String::from_utf8(output.stderr)?));
    }
}

/// Add metric values
/// See [Juju Metrics](https://jujucharms.com/docs/2.0/developer-metrics) for more
/// information
/// May only be called from the collect-metrics hook
/// # Failures
/// Returns stderr if the add_metric command fails
pub fn add_metric(key: &str, value: &str) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(format!("{}={}", key, value));

    let output = run_command("add-metric", &arg_list, false)?;
    return process_output(output);
}

/// Get the availability zone
/// # Failures
/// Returns stderr if the meter_status command fails
pub fn az_info() -> Result<String, JujuError> {
    let az = env::var("JUJU_AVAILABILITY_ZONE")?;
    return Ok(az);
}

/// Get the meter status, if running in the meter-status-changed hook
/// # Failures
/// Returns stderr if the meter_status command fails
pub fn meter_status() -> Result<String, JujuError> {
    let status = env::var("JUJU_METER_STATUS")?;
    return Ok(status);
}


/// Get the meter status information, if running in the meter-status-changed hook
/// # Failures
/// Returns stderr if the meter_info command fails
pub fn meter_info() -> Result<String, JujuError> {
    let info = env::var("JUJU_METER_INFO")?;
    return Ok(info);
}

/// This will reboot your juju instance.  Examples of using this are when a new kernel is installed
/// and the virtual machine or server needs to be rebooted to use it.
/// # Failures
/// Returns stderr if the reboot command fails
pub fn reboot() -> Result<i32, JujuError> {
    let output = run_command_no_args("juju-reboot", true)?;
    return process_output(output);
}
/// Charm authors may trigger this command from any hook to output what
/// version of the application is running. This could be a package version,
/// for instance postgres version 9.5. It could also be a build number or
/// version control revision identifier, for instance git sha 6fb7ba68.
/// # Failures
/// Returns stderr if the action_get command fails
pub fn application_version_set(version: &str) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(version.to_string());
    let output = run_command("application-version-set", &arg_list, false)?;
    return process_output(output);
}

/// action_get_all gets all values that are set
/// See [Juju Actions](https://jujucharms.com/docs/devel/authors-charm-actions) for more information
/// # Failures
/// Returns stderr if the action_get command fails
pub fn action_get_all() -> Result<HashMap<String, String>, JujuError> {
    let output = run_command_no_args("action-get", false)?;
    let values = String::from_utf8(output.stdout)?;
    let mut map: HashMap<String, String> = HashMap::new();

    for line in values.lines() {
        let parts: Vec<&str> = line.split(":").collect();
        if parts.len() == 2 {
            map.insert(parts[0].to_string(), parts[1].trim().to_string());
        }
    }
    return Ok(map);
}

/// action_get gets the value of the parameter at the given key
/// See [Juju Actions](https://jujucharms.com/docs/devel/authors-charm-actions) for more information
/// # Failures
/// Returns stderr if the action_get command fails
pub fn action_get(key: &str) -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(key.to_string());

    let output = run_command("action-get", &arg_list, false)?;
    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get the name of the currently executing action
/// # Failures
/// Returns JujuError if the environment variable JUJU_ACTION_NAME does not exist
pub fn action_name() -> Result<String, JujuError> {
    let name = env::var("JUJU_ACTION_NAME")?;
    return Ok(name);
}

/// Get the uuid of the currently executing action
/// # Failures
/// Returns JujuError if the environment variable JUJU_ACTION_UUID does not exist
pub fn action_uuid() -> Result<String, JujuError> {
    let uuid = env::var("JUJU_ACTION_UUID")?;
    return Ok(uuid);
}

/// Get the tag of the currently executing action
/// # Failures
/// Returns JujuError if the environment variable JUJU_ACTION_TAG does not exist
pub fn action_tag() -> Result<String, JujuError> {
    let tag = env::var("JUJU_ACTION_TAG")?;
    return Ok(tag);
}

/// action_set permits the Action to set results in a map to be returned at completion of
/// the Action.
/// See [Juju Actions](https://jujucharms.com/docs/devel/authors-charm-actions) for more
/// information
/// # Failures
/// Returns stderr if the action_set command fails
pub fn action_set(key: &str, value: &str) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(format!("{}={}", key, value));

    let output = run_command("action-set", &arg_list, false)?;
    return process_output(output);
}

/// See [Juju Actions](https://jujucharms.com/docs/devel/authors-charm-actions) for more
/// information
/// # Failures
/// Returns stderr if the action_fail command fails
pub fn action_fail(msg: &str) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(msg.to_string());

    let output = run_command("action-fail", &arg_list, false)?;
    return process_output(output);
}

/// This will return the private IP address associated with the unit.
/// It can be very useful for services that require communicating with the other units related
/// to it.
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn unit_get_private_addr() -> Result<IpAddr, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push("private-address".to_string());

    let output = run_command("unit-get", &arg_list, false)?;
    let private_addr: String = String::from_utf8(output.stdout)?;
    let ip = IpAddr::from_str(private_addr.trim())?;
    return Ok(ip);
}

/// This will return the public IP address associated with the unit.
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn unit_get_public_addr() -> Result<IpAddr, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push("public-address".to_string());

    let output = run_command("unit-get", &arg_list, false)?;
    let public_addr = String::from_utf8(output.stdout)?;
    let ip = IpAddr::from_str(public_addr.trim())?;
    return Ok(ip);
}

/// This will return a configuration item that corresponds to the key passed in
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn config_get(key: &str) -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(key.to_string());

    let output = run_command("config-get", &arg_list, false)?;
    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// config_get_all will return all configuration options as a HashMap<String,String>
/// # Failures
/// Returns a String of if the configuration options are not able to be transformed into a HashMap
pub fn config_get_all() -> Result<HashMap<String, String>, JujuError> {
    let mut values: HashMap<String, String> = HashMap::new();

    let arg_list: Vec<String> = vec!["--all".to_string()];
    let output = run_command("config-get", &arg_list, false)?;
    let output_str = String::from_utf8(output.stdout)?;
    //  Example output:
    // "brick_paths: /mnt/brick1 /mnt/brick2\ncluster_type: Replicate\n"
    //
    // For each line split at : and load the parts into the HashMap
    for line in output_str.lines() {
        if let Some(position) = memchr(b':', &line.as_bytes()) {
            values.insert(line[0..position].trim().to_string(),
                          line[position + 1..].trim().to_string());
        }
        // Skip blank lines or failed splits
    }

    return Ok(values);
}

/// This will expose a port on the unit.  The transport argument will indicate whether tcp or udp
/// should be exposed
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn open_port(port: usize, transport: Transport) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    let port_string = format!("{}/{}", port.to_string(), transport.to_string());

    arg_list.push(port_string);
    let output = run_command("open-port", &arg_list, false)?;
    return process_output(output);
}

/// This will hide a port on the unit.  The transport argument will indicate whether tcp or udp
/// should be exposed
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn close_port(port: usize, transport: Transport) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    let port_string = format!("{}/{}", port.to_string(), transport.to_string());

    arg_list.push(port_string);
    let output = run_command("close-port", &arg_list, false)?;
    return process_output(output);
}

/// Set relation information for the current unit
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_set(key: &str, value: &str) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    let arg = format!("{}={}", key.clone(), value);

    arg_list.push(arg);
    let output = run_command("relation-set", &arg_list, false)?;
    return process_output(output);
}
/// Sets relation information using a specific relation ID. Used outside of relation hooks
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_set_by_id(key: &str, value: &str, id: &Relation) -> Result<String, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();

    arg_list.push(format!("-r {}:{}", id.name, id.id.to_string()));
    arg_list.push(format!("{}={}", key, value).to_string());

    let output = run_command("relation-set", &arg_list, false)?;
    let relation = String::from_utf8(output.stdout)?;
    return Ok(relation);
}

/// Get relation information for the current unit
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_get(key: &str) -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(key.to_string());
    let output = run_command("relation-get", &arg_list, false)?;
    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get relation information for a specific unit
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_get_by_unit(key: &str, unit: &Relation) -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(key.to_string());
    arg_list.push(format!("{}/{}", unit.name, unit.id.to_string()));

    let output = run_command("relation-get", &arg_list, false)?;
    let relation = String::from_utf8(output.stdout)?.trim().to_string();
    if relation.is_empty() {
        Ok(None)
    } else {
        Ok(Some(relation))
    }
}

/// Get relation information using a specific relation ID. Used outside of relation hooks
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_get_by_id(key: &str,
                          id: &Relation,
                          unit: &Relation)
                          -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();

    arg_list.push(format!("-r {}:{}", id.name, id.id.to_string()));
    arg_list.push(format!("{}", key.to_string()));
    arg_list.push(format!("{}/{}", unit.name, unit.id.to_string()));

    let output = run_command("relation-get", &arg_list, false)?;
    let relation = String::from_utf8(output.stdout)?.trim().to_string();
    if relation.is_empty() {
        Ok(None)
    } else {
        Ok(Some(relation))
    }
}

/// Returns a list of all related units
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_list() -> Result<Vec<Relation>, JujuError> {
    let mut related_units: Vec<Relation> = Vec::new();

    let output = run_command_no_args("relation-list", false)?;
    let output_str = String::from_utf8(output.stdout)?;

    log(&format!("relation-list output: {}", output_str),
        Some(LogLevel::Debug));

    for line in output_str.lines() {
        let v: Vec<&str> = line.split('/').collect();
        let id: usize = v[1].parse::<usize>()?;
        let r: Relation = Relation {
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

/// Returns a list of all related units for the supplied identifier
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn relation_list_by_id(id: &Relation) -> Result<Vec<Relation>, JujuError> {
    let mut related_units: Vec<Relation> = Vec::new();
    let mut arg_list: Vec<String> = Vec::new();

    arg_list.push(format!("-r {}:{}", id.name, id.id.to_string()));

    let output = run_command("relation-list", &arg_list, false)?;
    let output_str = String::from_utf8(output.stdout)?;

    log(&format!("relation-list output: {}", output_str),
        Some(LogLevel::Debug));

    for line in output_str.lines() {
        let v: Vec<&str> = line.split('/').collect();
        let id: usize = v[1].parse::<usize>()?;
        let r: Relation = Relation {
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

pub fn relation_ids() -> Result<Vec<Relation>, JujuError> {
    let mut related_units: Vec<Relation> = Vec::new();
    let output = run_command_no_args("relation-ids", false)?;
    let output_str: String = String::from_utf8(output.stdout)?;
    log(&format!("relation-ids output: {}", output_str),
        Some(LogLevel::Debug));

    for line in output_str.lines() {
        let v: Vec<&str> = line.split(':').collect();
        let id: usize = v[1].parse::<usize>()?;
        let r: Relation = Relation {
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

/// Gets the relation IDs by their identifier
/// # Failures
/// Will return a String of the stderr if the call fails

pub fn relation_ids_by_identifier(id: &str) -> Result<Vec<Relation>, JujuError> {
    let mut related_units: Vec<Relation> = Vec::new();
    let mut arg_list: Vec<String> = Vec::new();

    arg_list.push(id.to_string());

    let output = run_command("relation-ids", &arg_list, false)?;
    let output_str: String = String::from_utf8(output.stdout)?;
    log(&format!("relation-ids output: {}", output_str),
        Some(LogLevel::Debug));

    for line in output_str.lines() {
        let v: Vec<&str> = line.split(':').collect();
        let id: usize = v[1].parse::<usize>()?;
        let r: Relation = Relation {
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return Ok(related_units);
}

/// Set the status of your unit to indicate to the Juju if everything is ok or something is wrong.
/// See the Status enum for information about what can be set.
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn status_set(status: Status) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push(status.status_type.to_string());
    arg_list.push(status.message);

    let output = run_command("status-set", &arg_list, false)?;
    return process_output(output);
}

/// Retrieve the previously set juju workload state
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn status_get() -> Result<String, JujuError> {
    let output = run_command_no_args("status-get", false)?;
    return Ok(String::from_utf8(output.stdout)?);
}

/// If storage drives were allocated to your unit this will get the path of them.
/// In the storage-attaching hook this will tell you the location where the storage
/// is attached to.  IE: /dev/xvdf for block devices or /mnt/{name} for filesystem devices
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn storage_get_location() -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push("location".to_string());
    let output = run_command("storage-get", &arg_list, false)?;
    let stdout = String::from_utf8(output.stdout)?.trim().to_string();
    if stdout.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stdout))
    }
}

/// Return the location of the mounted storage device.  The mounted
/// storage devices can be gotten by calling storage_list() and
/// then passed into this function to get their mount location.
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn storage_get(name: &str) -> Result<Option<String>, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    arg_list.push("-s".to_string());
    arg_list.push(name.to_string());
    arg_list.push("location".to_string());
    let output = run_command("storage-get", &arg_list, false)?;
    let stdout = String::from_utf8(output.stdout)?.trim().to_string();
    if stdout.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stdout))
    }
}

/// Used to list storage instances that are attached to the unit.
/// The names returned may be passed through to storage_get
/// # Failures
/// Will return a String of the stderr if the call fails
pub fn storage_list() -> Result<String, JujuError> {
    let output = run_command_no_args("storage-list", false)?;
    return Ok(String::from_utf8(output.stdout)?);
}

/// Call this to process your cmd line arguments and call any needed hooks
/// # Examples
/// ```
///     extern crate juju;
///     extern crate log;
///
///     fn config_changed()->Result<(), String>{
///         //Do nothing
///         return Ok(());
///    }
///
///     let mut hook_registry: Vec<juju::Hook> = Vec::new();
///
///     //Register our hooks with the Juju library
///     hook_registry.push(juju::Hook{
///         name: "config-changed".to_string(),
///         callback: config_changed,
///     });
///     let result =  juju::process_hooks(hook_registry);
///
///     if result.is_err(){
///         juju::log(&format!("Hook failed with error: {:?}", result.err()),
///         Some(log::LogLevel::Error));
///     }
/// ```
///
pub fn process_hooks(registry: Vec<Hook>) -> Result<(), String> {
    let hook_name = match charmhelpers::core::hookenv::hook_name() {
        Some(s) => s,
        _ => "".to_string(),
    };

    for hook in registry {
        if hook_name.contains(&hook.name) {
            return (hook.callback)();
        }
    }
    return Err(format!("Warning: Unknown callback for hook {}", hook_name));
}

/// Juju leader get value(s)
/// # Failures
/// Will return stderr as a String if the function fails to run
pub fn leader_get(attribute: Option<String>) -> Result<Option<String>, JujuError> {
    let arg_list: Vec<String>;
    match attribute {
        Some(a) => arg_list = vec![a],
        None => arg_list = vec!['-'.to_string()],
    };
    let output = run_command("leader-get", &arg_list, false)?;
    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}


/// Juju leader set value(s)
/// # Failures
/// Will return stderr as a String if the function fails to run
pub fn leader_set(settings: HashMap<String, String>) -> Result<i32, JujuError> {
    let mut arg_list: Vec<String> = Vec::new();
    for (key, value) in settings {
        arg_list.push(format!("{}={}", key, value));
    }

    let output = run_command("leader-set", &arg_list, false)?;
    return process_output(output);
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
///     println!("Failed to run.  Error was {:?}", e);
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
pub fn is_leader() -> Result<bool, JujuError> {
    let output = run_command_no_args("is-leader", false)?;
    let output_str: String = String::from_utf8(output.stdout)?;
    match output_str.trim().as_ref() {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Ok(false),
    }
}

fn run_command_no_args(command: &str, as_root: bool) -> Result<std::process::Output, JujuError> {
    if as_root {
        let mut cmd = std::process::Command::new("sudo");
        let output = cmd.output()?;
        return Ok(output);
    } else {
        let mut cmd = std::process::Command::new(command);
        let output = cmd.output()?;
        return Ok(output);
    }
}

fn run_command(command: &str,
               arg_list: &Vec<String>,
               as_root: bool)
               -> Result<std::process::Output, JujuError> {
    if as_root {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg(command);
        for arg in arg_list {
            cmd.arg(&arg);
        }
        let output = cmd.output()?;
        return Ok(output);
    } else {
        let mut cmd = std::process::Command::new(command);
        for arg in arg_list {
            cmd.arg(&arg);
        }
        let output = cmd.output()?;
        return Ok(output);
    }
}
