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
    relation_type: String,
    /// The relation ID for the current relation hook
    relation_id: usize,
    /// Local unit ID
    unit: String,
    /// relation data for all related units
    relations: HashMap<String,String>,
}
impl Context{
    //Gets a context that's filled out from the env variables
    fn new_from_env() -> Context{
        let mut relations: HashMap<String,String> = HashMap::new();
        let relation_type = env::var("JUJU_RELATION").unwrap_or("".to_string());
        let relation_id = env::var("JUJU_RELATION_ID").unwrap_or("0".to_string())
            .parse::<usize>().unwrap();
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

pub fn log(msg: &String){
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(msg.clone());
    run_command("juju-log", &arg_list, false);
}

pub fn reboot(){
    run_command_no_args("juju-reboot", true);
}

pub fn unit_get_private_addr() ->String{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("private-address".to_string());
    let output = run_command("unit-get", &arg_list, false);
    return String::from_utf8(output.stdout).unwrap();
}

pub fn unit_get_public_addr() ->String{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push("public-address".to_string());
    let output = run_command("unit-get", &arg_list, false);
    return String::from_utf8(output.stdout).unwrap();
}

pub fn config_get(key: &String) ->String{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(key.clone());
    let output = run_command("config-get", &arg_list, false);
    return String::from_utf8(output.stdout).unwrap();
}

pub fn open_port(port: usize, transport: Transport){
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string(), transport.to_string());

    arg_list.push(port_string);
    run_command("open-port", &arg_list, false);
}

pub fn close_port(port: usize, transport: Transport){
    let mut arg_list: Vec<String>  = Vec::new();
    let port_string = format!("{}/{}", port.to_string() , transport.to_string());

    arg_list.push(port_string);
    run_command("close-port", &arg_list, false);
}

pub fn relation_set(key: &String, value: &String){
    let mut arg_list: Vec<String>  = Vec::new();
    let arg = key.clone() + &"=" + value;

    arg_list.push(arg);
    run_command("relation-set", &arg_list, false);
}

pub fn relation_get(key: &String) -> String{
    let mut arg_list: Vec<String>  = Vec::new();
    arg_list.push(key.clone());
    let output = run_command("relation-get", &arg_list, false);
    return String::from_utf8(output.stdout).unwrap();
}

pub fn relation_get_by_unit(key: &String, unit: &Relation) -> String{
    let mut arg_list: Vec<String>  = Vec::new();
    //arg_list.push("-r".to_string());
    arg_list.push(key.clone());
    arg_list.push(format!("{}/{}", unit.name , unit.id.to_string()));
    let output = run_command("relation-get", &arg_list, false);
    return String::from_utf8(output.stdout).unwrap();
}

pub fn relation_list() ->Vec<Relation>{
    let mut related_units: Vec<Relation> = Vec::new();
    let output = run_command_no_args("relation-list", false);
    let output_str =  String::from_utf8(output.stdout).unwrap();
    log(&format!("relation-list output: {}", output_str));

    for line in output_str.lines(){
        let v: Vec<&str> = line.split('/').collect();
        let id: usize = v[1].parse::<usize>().unwrap();
        let r: Relation = Relation{
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return related_units;
}

pub fn relation_ids() ->Vec<Relation>{
    let mut related_units: Vec<Relation> = Vec::new();
    let output = run_command_no_args("relation-ids", false);
    let output_str =  String::from_utf8(output.stdout).unwrap();
    log(&format!("relation-ids output: {}", output_str));

    for line in output_str.lines(){
        let v: Vec<&str> = line.split(':').collect();
        let id: usize = v[1].parse::<usize>().unwrap();
        let r: Relation = Relation{
            name: v[0].to_string(),
            id: id,
        };
        related_units.push(r);
    }
    return related_units;
}

fn run_command_no_args(command: &str, as_root: bool)-> std::process::Output{
    if as_root{
        let mut cmd = std::process::Command::new("sudo");
        println!("Running command: {:?}", cmd);
        let output = cmd.output().unwrap_or_else(|e| { panic!("failed to execute process: {} ", e)});
        return output;
    }else{
       let mut cmd = std::process::Command::new(command);
        println!("Running command: {:?}", cmd);
        let output = cmd.output().unwrap_or_else(|e| { panic!("failed to execute process: {} ", e)});
        return output;
    }
}

fn run_command(command: &str, arg_list: &Vec<String>, as_root: bool) -> std::process::Output{
    if as_root{
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg(command);
        for arg in arg_list{
            cmd.arg(&arg);
        }
        println!("Running command: {:?}", cmd);
        let output = cmd.output().unwrap_or_else(|e| { panic!("failed to execute process: {} ", e)});
        return output;
    }else{
       let mut cmd = std::process::Command::new(command);
        for arg in arg_list{
            cmd.arg(&arg);
        }
        println!("Running command: {:?}", cmd);
        let output = cmd.output().unwrap_or_else(|e| { panic!("failed to execute process: {} ", e)});
        return output;
    }
}
