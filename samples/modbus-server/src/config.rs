use std::{
    fs::File,
    io::Read,
    path::Path,
};

use crate::Table;

#[derive(Deserialize, Clone, Copy, PartialEq)]
pub enum AliasType {
    Default,
    Boolean,
    Byte,
    SByte,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float,
    Double,
}

impl AliasType {
    /// Returns the size of the type in number of registers
    pub fn size_in_words(&self) -> u16 {
        match self {
            Self::Default | Self::Boolean | Self::Byte | Self::SByte | Self::UInt16 | Self::Int16 => 1,
            Self::UInt32 => 2,
            Self::Int32 => 2,
            Self::UInt64 => 4,
            Self::Int64 => 4,
            Self::Float => 2,
            Self::Double => 4
        }
    }
}

fn default_as_u16() -> AliasType {
    AliasType::Default
}

fn default_as_false() -> bool {
    false
}

#[derive(Deserialize, Clone)]
pub struct Alias {
    pub name: String,
    pub number: u16,
    #[serde(default = "default_as_u16")]
    pub data_type: AliasType,
    #[serde(default = "default_as_false")]
    pub writable: bool,
}

#[derive(Deserialize, Clone)]
pub struct TableConfig {
    pub base_address: u16,
    pub count: u16,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            base_address: 0u16,
            count: 0u16,
        }
    }
}

impl TableConfig {
    pub fn valid(&self) -> bool {
        if self.base_address >= 9998 || self.base_address + self.count > 9999 {
            false
        } else {
            true
        }
    }

    pub fn in_range(&self, addr: u16) -> bool {
        addr >= self.base_address && addr < self.base_address + self.count
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub slave_address: String,
    pub read_interval: u32,
    pub input_coils: TableConfig,
    pub output_coils: TableConfig,
    pub input_registers: TableConfig,
    pub output_registers: TableConfig,
    pub aliases: Option<Vec<Alias>>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ()> {
        if let Ok(mut f) = File::open(path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                let config = serde_yaml::from_str(&s);
                if let Ok(config) = config {
                    Ok(config)
                } else {
                    println!("Cannot deserialize configuration from {}", path.to_string_lossy());
                    Err(())
                }
            } else {
                println!("Cannot read configuration file {} to string", path.to_string_lossy());
                Err(())
            }
        } else {
            println!("Cannot open configuration file {}", path.to_string_lossy());
            Err(())
        }
    }

    pub fn valid(&self) -> bool {
        let mut valid = true;
        if self.slave_address.is_empty() {
            println!("No slave IP address specified");
            valid = false;
        }
        if !self.input_coils.valid() {
            println!("Input coil addresses are out of range");
            valid = false;
        }
        if !self.output_coils.valid() {
            println!("Output coil addresses are out of range");
            valid = false;
        }
        if !self.input_registers.valid() {
            println!("Input register addresses are out of range");
            valid = false;
        }
        if !self.output_registers.valid() {
            println!("Input register addresses are out of range");
            valid = false;
        }
        if let Some(ref aliases) = self.aliases {
            let set: std::collections::HashSet<&str> = aliases.iter().map(|a| a.name.as_ref()).collect::<_>();
            if set.len() != aliases.len() {
                println!("Aliases contains duplicate names");
                valid = false;
            }
            aliases.iter().for_each(|a| {
                // Check the register is addressable
                let number = a.number;
                let (table, addr) = Table::table_from_number(number);
                let in_range = match table {
                    Table::OutputCoils => self.output_coils.in_range(addr),
                    Table::InputCoils => self.input_coils.in_range(addr),
                    Table::InputRegisters => self.input_registers.in_range(addr),
                    Table::OutputRegisters => self.output_registers.in_range(addr),
                };
                if !in_range {
                    println!("Alias {} has an out of range register of {}, check base address and count of the corresponding table", a.name, number);
                    valid = false;
                }

                if table == Table::OutputCoils || table == Table::InputCoils {
                    // Coils
                    // Coils must be booleans
                    if a.data_type != AliasType::Boolean && a.data_type != AliasType::Default {
                        println!("Alias {} for coil must be of type Boolean", a.name);
                        valid = false;
                    }
                } else {
                    // Check that the size of the type does not exceed the range
                    let cnt = a.data_type.size_in_words();
                    let end = number + cnt;
                    let max = if table == Table::InputRegisters { 39999 } else { 49999 };
                    if end > max {
                        println!("Alias {} starts with number {} but has a data type whose word size {} that exceeds the table max of {}", a.name, number, cnt, max);
                        valid = false;
                    }
                }
            });
        }
        valid
    }
}
