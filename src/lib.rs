use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub enum Transport {
    Tcp,
    Udp,
}

impl Transport {
    fn to_string(self) -> String {
        match self {
            Transport::Tcp => "tcp".to_string(),
            Transport::Udp => "udp".to_string(),
        }
    }
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
    //Gets a context that's filled out from the env variables
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
    pub name: String,
    pub id: usize
}

pub fn log(msg: &String){//->Result<i32,String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(msg.clone());

    //Ignoring errors if they happen.
    //TODO: should this return success/failure?  It makes the code ugly
    run_command("juju-log", &arg_list, false).is_ok();
}

pub fn reboot()->Result<i32,String>{
    let output = try!(run_command_no_args("juju-reboot", true).map_err(|e| e.to_string()));

    let status = output.status;
    match status.code(){
        Some(v) => Ok(v),
        None => Err(try!(String::from_utf8(output.stderr).map_err(|e| e.to_string()))),
    }
}

pub fn unit_get_private_addr() ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("private-address".to_string());
    let output = try!(run_command("unit-get", &arg_list, false).map_err(|e| e.to_string()));
    let private_addr: String = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(private_addr.trim().to_string());
}

pub fn unit_get_public_addr() ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("public-address".to_string());
    let output = try!(run_command("unit-get", &arg_list, false).map_err(|e| e.to_string()));
    let public_addr = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(public_addr.trim().to_string());
}

pub fn config_get(key: &String) ->Result<String, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(key.clone());
    let output = try!(run_command("config-get", &arg_list, false).map_err(|e| e.to_string()));
    let value = try!(String::from_utf8(output.stdout).map_err(|e| e.to_string()));
    return Ok(value.trim().to_string());
}

pub fn open_port(port: usize, transport: Transport)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string(), transport.to_string());

    arg_list.push(port_string);
    let output = try!(run_command("open-port", &arg_list, false).map_err(|e| e.to_string()));

    let status = output.status;
    match status.code(){
        Some(v) => Ok(v),
        None => Err(try!(String::from_utf8(output.stderr).map_err(|e| e.to_string()))),
    }
}

pub fn close_port(port: usize, transport: Transport)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string() , transport.to_string());

    arg_list.push(port_string);
    let output = try!(run_command("close-port", &arg_list, false).map_err(|e| e.to_string()));

    let status = output.status;
    match status.code(){
        Some(v) => Ok(v),
        None => Err(try!(String::from_utf8(output.stderr).map_err(|e| e.to_string()))),
    }
}

pub fn relation_set(key: &str, value: &str)->Result<i32, String>{
    let mut arg_list: Vec<String>  = Vec::new();
    let arg = format!("{}={}", key.clone(), value);

    arg_list.push(arg);
    let output = try!(run_command("relation-set", &arg_list, false).map_err(|e| e.to_string()));
    let status = output.status;
    match status.code(){
        Some(v) => Ok(v),
        None => Err(try!(String::from_utf8(output.stderr).map_err(|e| e.to_string()))),
    }
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


//Call back any functions that are registered
/*
pub fn process(args: Vec<String>){

}

///
pub fn register_hook<F: Fn()>(func: F){

}
*/

//Returns true/false if this unit is the leader
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
