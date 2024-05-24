use std::fs::{read,OpenOptions};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::str::FromStr;
use std::string::ParseError;
use std::path::PathBuf;
use std::env;
use colored::*;
use inquire::{CustomType,Text,Select};
use chrono::{Utc,FixedOffset,NaiveDateTime};

#[derive(Default,Debug)]
struct Config {
    history_file_path: PathBuf,
    objectives: Option<Vec<(String,f64)>>,
    trim_size: Option<usize>,
    sep_size: Option<usize>
}

struct Registry {
    date: String,
    money: f64,
    desc: String
}

fn get_current_date() -> String {
    Utc::now().with_timezone(&FixedOffset::west_opt(10800).unwrap()).format("%d/%m/%Y").to_string()
}

fn date_str_to_timestamp(date:&str) -> i64 {
    NaiveDateTime::parse_from_str(&format!("{} 00:00:00",date), "%d/%m/%Y %H:%M:%S").expect("Falha ao processar data !").timestamp()
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

fn add_registry(config: Config) {
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
        .open(config.history_file_path)
        .unwrap();
    
    file.write(format!("{}:{}:{}\n",get_current_date(),value,desc).as_bytes()).expect("Falha ao escrever arquivo.");
    println!("{} Registro adicionado.",">".bright_green())
}

fn read_registry(config: Config,filters: (i64,i64)) {
    let file = match read(config.history_file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("{}\nVocê deve adicionar pelo menos um registro para criar o aquivo.","Arquivo de log não existente !".bright_red());
            return;
        }
    };

    let log = String::from_utf8(file).unwrap();

    let mut sum = 0.0;
    let mut registries: Vec<Registry> = log.lines().map(|f|Registry::from_str(f).expect("Falha ao passar linha do log.")).collect();
    if filters != (0,i64::MAX) {
        registries = registries.into_iter().filter(|f|{let tp = date_str_to_timestamp(&f.date); tp>filters.0 && tp<filters.1}).collect();
    }
    let trim_hide = registries.len().checked_sub(config.trim_size.unwrap_or(50)).unwrap_or(0);
    if trim_hide > 0 {
        println!("↑\n| Ocultando {} registros",trim_hide);
    }
    for (i,reg) in registries.iter().enumerate() {
        //trim print size
        if i >= trim_hide {
            let print_money = if reg.money > 0.0 {
                format!("↑${:.2}",reg.money).bright_green()
            } else {
                format!("↓${:.2}",reg.money*-1.0).bright_red()
            };
                println!("[{}] {}: {}",reg.date,print_money,reg.desc);
        }

        sum += reg.money;
    }
    if config.objectives.is_some() {
        println!("{}","=".repeat(config.sep_size.unwrap_or(50)));
        for objective in config.objectives.unwrap() {
            let perc = (sum/objective.1)*100.0;
            let mut perc_str = format!("{:.2}%",perc);
            if perc > 100.0 {
               perc_str = perc_str.bright_green().blink().to_string();
            } else {
                perc_str = perc_str.bright_yellow().to_string();
            }
            println!("{}: ${} ({})",objective.0,objective.1,perc_str)
        }
    }
    println!("{}\nTotal: ${}","=".repeat(config.sep_size.unwrap_or(50)),format!("{:.2}",sum).to_string().bright_cyan())
}

fn main() {
    //oppening config file
    let config_file = match read(".fexcel.conf") {
        Ok(file) => file,
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                panic!("Erro desconhecido: {}",e);
            }
            println!("Sem arquivo de configuração !\nGerando arquivo padrão: .fexcel.conf");
            let defaut_config = include_bytes!("..\\default.conf");
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(".fexcel.conf")
                .unwrap();
            file.write(defaut_config).expect("Falha ao escver arquivo !");
            defaut_config.to_vec()
        }
    };

    let mut config = Config::default();

    //reading config file
    for line in String::from_utf8(config_file).unwrap().lines() {
        let (conf,param) = line.split_once("=").unwrap();
        match conf {
            "HISTORY_FILE" => {config.history_file_path = param.parse().expect("Erro ao parse !")},
            "OBJECTIVES" => {
                let mut objectives: Vec<(String,f64)> = Vec::new();
                for obj in param.split(",") {
                    if let Some((desc,val)) = obj.split_once(":") {
                        objectives.push((desc.parse().expect("Erro ao parse !"),val.parse().expect("Valor do objetivo incorreto !")))
                    }
                }
                config.objectives = Some(objectives)
            },
            "TRIM" => {
                config.trim_size = Some(param.parse().expect("Erro ao parse !"))
            }
            "SEP_SIZE" => {
                config.sep_size = Some(param.parse().expect("Erro ao parse !"))
            }
            _ => ()
        }
    }

    //parse args
    let mut filters: (i64,i64) = (0,i64::MAX);
    let args: Vec<String> = env::args().collect();
    for (i,arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--ss" => {
                filters.0 = date_str_to_timestamp(args.get(i+1).expect("Argumento errado !")); //start
            },
            "--to" => {
                filters.1 = date_str_to_timestamp(args.get(i+1).expect("Argumento errado !")); //end
            },
            "--t" => {

                filters.0 = date_str_to_timestamp(&get_current_date()) - (args.get(i+1).expect("Argumento errado !").parse::<i64>().unwrap()*86400); //current date - X in days
            }
            "--help" => {
                println!(include_str!("..\\help.txt"));
                return;
            }
            _ => {}
        }
    }

    if filters != (0,i64::MAX) {
        read_registry(config,filters)
    } else {
        let opts = vec!["Ver registros","Adicionar registro"];
        let sel = Select::new("",opts.to_owned())
            .with_help_message("↑↓ para mover")
            .prompt()
            .unwrap();
        if sel == opts[0] {
            read_registry(config,filters)
        } else if sel == opts[1] {
            add_registry(config)
        }
    }
}
