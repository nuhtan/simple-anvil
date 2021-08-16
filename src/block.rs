use core::{fmt, panic};

use nbt::Value;

#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    namespace: String,
    pub id: String,
}

impl Block {
    pub fn new(namespace: String, block_id: Option<String>) -> Block {
        match block_id {
            Some(id) => return Block { namespace, id },
            None => {
                let idn = namespace.clone();
                return Block { namespace, id: idn };
            }
        }
    }

    pub fn name(self) -> String {
        let mut name = self.namespace;
        name += ":";
        name += self.id.as_str();
        return name;
    }

    pub fn from_name(name: String) -> Block {
        let temp: Vec<&str> = name.split(":").collect();
        return Block {
            namespace: temp[0].to_owned(),
            id: temp[1].to_owned(),
        };
    }

    pub fn from_palette(tag: &Value) -> Block {
        let tag = if let Value::Compound(t) = tag {
            t
        } else {
            panic!("Tag passed from palette is not compound")
        };
        let name = if let Value::String(n) = tag.get("Name").unwrap() {
            n
        } else {
            panic!("Palette tag missing name?")
        };
        return Block::from_name(name.to_string());
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}
