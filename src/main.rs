use std::fs::{read,OpenOptions};
use std::io::prelude::*;
use std::str::FromStr;
use std::string::ParseError;
use colored::*;
use inquire::{CustomType,Text,Select};
use chrono::Local;

const HISTORY_FILE: &str = "history.log";

struct Registry {
    date: String,
    money: f64,
    desc: String
}

macro_rules! get_current_date {
    () => {
        Local::now().format("%d/%m/%Y").to_string()
    };
}

impl FromStr for Registry {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (date,values) = s.split_once(":").unwrap();
        let (money,desc) = values.split_once(":").unwrap();
        
        Ok(
            Registry {
                date: date.parse().unwrap(),
                money: money.parse().unwrap(),
                desc: desc.parse().unwrap()
            }
        )
    }
}

fn add_registry() {
    let value: f64 = CustomType::new("Valor:")
        .with_formatter(&|i: f64| format!("${}", i))
        .with_error_message("Favor adicionar um número valido")
        .with_help_message("Negativo para gastos, use ponto para centavos")
        .prompt()
        .unwrap();
    
    let desc = Text::new("Descrição:")
        .with_help_message("Descrição para o registro")
        .prompt()
        .unwrap();
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(HISTORY_FILE)
        .unwrap();
    
    file.write(format!("{}:{}:{}\n",get_current_date!(),value,desc).as_bytes()).expect("Falha ao escrever arquivo.");
    println!("{} Registro adicionado.",">".bright_green())
}

fn read_registry() {
    let file = match read(HISTORY_FILE) {
        Ok(file) => file,
        Err(_) => {
            println!("{}\nVocê deve adicionar pelo menos um registro para criar o aquivo.","Arquivo de log não existente !".bright_red());
            return;
        }
    };

    let log = String::from_utf8(file).unwrap();

    let mut sum = 0.0;
    for line in log.lines() {
        let reg = Registry::from_str(line).expect("Falha ao passar linha do log.");
        let print_money = if reg.money > 0.0 {
            format!("↑${:.2}",reg.money).bright_green()
        } else {
            format!("↓${:.2}",reg.money*-1.0).bright_red()
        };
        println!("[{}] {}: {}",reg.date,print_money,reg.desc);
        sum += reg.money;
    }
    println!("{}\nTotal: ${}","=".repeat(50),format!("{:.2}",sum).to_string().bright_cyan())
}

fn main() {
    let opts = vec!["Ver registros","Adicionar registro"];
    let sel = Select::new("",opts.to_owned())
        .with_help_message("↑↓ para mover")
        .prompt()
        .unwrap();
    if sel == opts[0] {
        read_registry()
    } else if sel == opts[1] {
        add_registry()
    }
}
