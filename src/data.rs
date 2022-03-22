use crate::config::CONFIG;
use cached::{Cached, TimedSizedCache};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub token: String,
    pub key: String,
    pub shards: u64,
    pub data: String,
    pub infos: Infos,
}

#[derive(Deserialize, Serialize)]
pub struct Infos {
    pub name: String,
    pub prefix: String,
    pub activity: String,
}

pub static DATA: Lazy<Mutex<TimedSizedCache<u64, Yaml>>> = Lazy::new(|| {
    Mutex::new(TimedSizedCache::with_size_and_lifespan(
        CONFIG.cache.size,
        CONFIG.cache.time * 60,
    ))
});
pub static EMPTY: Lazy<Yaml> = Lazy::new(|| Yaml::from_str("").clone());

fn load(path_id: &String) -> Option<Yaml> {
    let path = to_path(path_id);
    if !Path::new(&path).exists() {
        return None;
    }
    let file = fs::read_to_string(&path).unwrap();
    Some(YamlLoader::load_from_str(&file).unwrap()[0].clone())
}

fn to_path(path_id: &String) -> String {
    if path_id.ends_with(".yaml") {
        path_id.clone()
    } else {
        format!("{}{}.yaml", &CONFIG.data, path_id)
    }
}

pub fn dump(yaml: &Yaml) -> String {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&yaml).unwrap();
    if out_str.contains(":") {
        out_str
    } else {
        "".to_string()
    }
    
}

fn write(id: &u64, out_str: &str) {
    if out_str.contains(":") {
        fs::write(to_path(&id.to_string()), out_str).unwrap();
    } else {
        fs::remove_file(to_path(&id.to_string())).unwrap();
    }
    DATA.lock().unwrap().cache_remove(id);
}

pub fn remove(id: &u64, key: &str) -> bool {
    let yaml = get(id);
    if !yaml[key].is_badvalue() {
        let mut yaml_str = dump(&yaml);
        let mut delete = false;
        let target = format!("{}:", key);
        let target_dq = format!("\"{}\":", key);
        if !yaml_str.ends_with("\n") {
            yaml_str += "\n"
        }
        let mut lines: Vec<&str> = yaml_str.lines().collect();

        let mut i = 0;
        let mut to_be_removed: Vec<usize> = vec![];
        for v in lines.clone() {
            if v.starts_with(&target) || v.starts_with(&target_dq) {
                to_be_removed.push(i);
                if let Yaml::String(_) = yaml[key] {} else {
                    delete = true;
                }
            } else if delete {
                if v.starts_with(" ") {
                    to_be_removed.push(i);
                } else {
                    break;
                }
            }
            i += 1;
        }

        i = 0;
        lines.retain(|_|{
            let a = !to_be_removed.contains(&i);
            i += 1;
            a
        });

        //Just in case
        let new_yaml = YamlLoader::load_from_str(&lines.join("\n")).unwrap();
        write(id, &dump(&new_yaml[0]));
        return true;
    }
    false
}

pub fn new(id: &u64, content: &str) -> Result<(), String> {
    let res = YamlLoader::load_from_str(content);
    if let Err(err) = res {
        return Err(err.to_string());
    } else {
        let doc = res.unwrap();
        if doc.len() != 1 {
            return Err("Less or too much document".to_string());
        }
        match verify(&doc[0]) {
            Ok(res) => {
                let yaml = get(id);
                for i in res {
                    if !yaml[&i as &str].is_badvalue() {
                        return Err("Already exists".to_string());
                    }
                }
                let mut dumped_yaml = dump(&yaml);
                if !dumped_yaml.contains(":") {
                    dumped_yaml = "".to_string();
                }

                //Just in case
                let new_yaml = YamlLoader::load_from_str(&format!(
                    "{}\n{}",
                    dumped_yaml,
                    dump(&doc[0]).replace("---\n", "")
                ))
                .unwrap();
                write(id, &dump(&new_yaml[0]));
            }
            Err(err) => return Err(err.to_string()),
        }
    }
    Ok(())
}

fn verify<'a>(request: &Yaml) -> Result<Vec<String>, &'a str> {
    let mut res = vec![];
    for (key, value) in request.as_hash().unwrap() {
        res.push(key.as_str().unwrap().to_string());
        match value {
            Yaml::String(_) => {}
            Yaml::Hash(_) => {
                match value["trigger"] {
                    Yaml::String(_) => {}
                    Yaml::Hash(_) => {
                        let mut uid_or_content = false;
                        match &value["trigger"]["uid"] {
                            Yaml::Array(arr) => {
                                for i in arr {
                                    match i {
                                        Yaml::Integer(_) => uid_or_content = true,
                                        _ => return Err("Uid trigger must be int"),
                                    }
                                }
                            }
                            Yaml::Integer(_) => uid_or_content = true,
                            Yaml::BadValue => {}
                            _ => return Err("Uid trigger must be int"),
                        }
                        match &value["trigger"]["content"] {
                            Yaml::String(_) => uid_or_content = true,
                            Yaml::Array(arr) => {
                                for i in arr {
                                    match i {
                                        Yaml::String(_) => uid_or_content = true,
                                        _ => return Err("Content trigger must be string"),
                                    }
                                }
                            }
                            Yaml::BadValue => {}
                            _ => return Err("Content trigger must be string"),
                        }
                        if !uid_or_content {
                            return Err("Trigger is required");
                        }
                    }
                    _ => return Err("Trigger is required"),
                }
                let mut return_react_or_js = false;
                match &value["return"] {
                    Yaml::String(_) => return_react_or_js = true,
                    Yaml::Array(arr) => {
                        for i in arr {
                            match i {
                                Yaml::String(_) => return_react_or_js = true,
                                _ => return Err("Return value must be string"),
                            }
                        }
                    }
                    Yaml::BadValue => {}
                    _ => return Err("Return value must be string"),
                }
                match &value["react"] {
                    Yaml::String(s) => {
                        if s.chars().count() != 1 {
                            return Err("React must be 1 char emoji");
                        }
                        return_react_or_js = true;
                    }
                    Yaml::Integer(_) => {
                        return_react_or_js = true;
                    }
                    Yaml::Array(arr) => {
                        for i in arr {
                            match i {
                                Yaml::Integer(_) => {
                                    return_react_or_js = true;
                                }
                                Yaml::String(s) => {
                                    if s.chars().count() != 1 {
                                        return Err("React must be 1 char emoji");
                                    }
                                }
                                _ => return Err("Emoji must be int or string"),
                            }
                        }
                    }
                    Yaml::BadValue => {}
                    _ => return Err("Emoji must be int or string"),
                }
                if let Yaml::String(_) = &value["js"] {
                    return_react_or_js = true;
                }
                if !return_react_or_js {
                    return Err("No action is defined");
                }
            }
            _ => return Err("Return value must be string"),
        }
    }
    Ok(res)
}

pub fn get(id: &u64) -> Yaml {
    if let Some(cache) = DATA.lock().unwrap().cache_get(id) {
        return (*cache).clone()
    }
    if let Some(yaml) = load(&id.to_string()) {
        return yaml;
    } else {
        DATA.lock().unwrap().cache_set(*id, EMPTY.clone());
        EMPTY.clone()
    }
}
