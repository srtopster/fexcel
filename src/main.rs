use std::fs::{OpenOptions,File};
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::path::PathBuf;
use std::env;
use std::collections::VecDeque;
use colored::*;
use inquire::{CustomType,Text,Select};
use chrono::{Utc,FixedOffset,NaiveDateTime};
use anyhow::{Result,anyhow,Context};
use ini::Ini;

const CONFIG_FILE_NAME: &str = ".fexcel.ini";

#[derive(Debug)]
struct Config {
    history_file_path: PathBuf,
    objectives: Option<Vec<(String,f64)>>,
    expenses: Option<Vec<(String,f64)>>,
    trim_size: usize,
    separator: String,
    separator_size: usize,
    value_padding: usize,
    rendered_separator: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            history_file_path: "history.log".into(),
            objectives: None,
            expenses: None,
            trim_size: 50,
            separator: "━".to_string(),
            separator_size: 50,
            value_padding: 9,
            rendered_separator: "━".to_string().repeat(50)
        }
    }
}

struct Args {
    filter: Option<FilterBounds>,
    help: bool
}

impl Default for Args {
    fn default() -> Self {
        Self {
            filter: None,
            help: false
        }
    }
}

#[derive(Debug)]
struct FilterBounds {
    lower: i64,
    upper: i64
}

struct Registry {
    date: String,
    money: f64,
    desc: String
}

impl FromStr for Registry {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (date,values): (&str,&str) = s.split_once(":").ok_or_else(||anyhow!("Falha ao split ':'"))?;
        let (money,desc): (&str,&str) = values.split_once(":").ok_or_else(||anyhow!("Falha ao split ':'"))?;
        
        Ok(
            Self {
                date: date.parse()?,
                money: money.parse()?,
                desc: desc.parse()?
            }
        )
    }
}

fn get_current_date() -> String {
    Utc::now().with_timezone(&FixedOffset::west_opt(10800).unwrap()).format("%d/%m/%Y").to_string()
}

fn date_str_to_timestamp(date:&str) -> Result<i64> {
    Ok(NaiveDateTime::parse_from_str(&format!("{} 00:00:00",date), "%d/%m/%Y %H:%M:%S").with_context(||anyhow!("Erro ao processar data: {}",date))?.timestamp())
}

fn config_parser(config_file_path: PathBuf) -> Result<Config> {
    //If configo file doesn't exist, write one
    if !config_file_path.try_exists()? {
        println!("{}","Sem arquivo de configuração !".bright_yellow());
        println!("{}","Gerando arquivo padrão: .fexcel.ini".bright_yellow());
        let mut file = File::create(&config_file_path)?; 
        file.write_all(include_bytes!("..\\fexcel_default.ini"))?;
    }

    let mut parsed_config = Config::default();
    let config_file = Ini::load_from_file(&config_file_path)?;

    //pega o caminho do arquivo de history
    parsed_config.history_file_path = config_file.section(None::<String>)
        .ok_or_else(||anyhow!("Falha ao pegar history_file na configuração ! Use: --help !"))?
        .get("history_file")
        .ok_or_else(||anyhow!("Falha ao pegar history_file na configuração ! Use: --help !"))?
        .parse()?;

    //Objectives
    if let Some(objectives_sec) = config_file.section(Some("objectives")) {
        let objectives = objectives_sec.iter()
            .map(|(k,v)| -> Result<(String,f64)>{
                Ok((
                    k.to_owned(),
                    v.parse::<f64>()
                        .with_context(|| format!("[Objectives] Erro ao fazer parse do valor '{}' para '{}' ! Use: --help", v, k))?
                ))
            })
            .collect::<Result<Vec<(String,f64)>>>()?; //Rust magia negra, tranforma um iter de Result<(k,v),Error> em um iterator de (k,v) e propaga o erro

        parsed_config.objectives = (!objectives.is_empty()).then_some(objectives)
    }

    //Expenses
    if let Some(expenses_sec) = config_file.section(Some("expenses")) {
        let expenses = expenses_sec.iter()
                .map(|(k,v)| -> Result<(String,f64)>{
                    Ok((
                        k.to_owned(),
                        v.parse::<f64>()
                            .with_context(|| format!("[Expenses] Erro ao fazer parse do valor '{}' para '{}' ! Use: --help", v, k))?
                    ))
                })
                .collect::<Result<Vec<(String,f64)>>>()?; //Rust magia negra, tranforma um iter de Result<(k,v),Error> em um iterator de (k,v) e propaga o erro

        parsed_config.expenses = (!expenses.is_empty()).then_some(expenses)
    }

    if let Some(style) = config_file.section(Some("style")) {
        parsed_config.trim_size = style.get("trim_size").map(str::parse).transpose()?.unwrap_or(parsed_config.trim_size);
        parsed_config.separator = style.get("separator").map(str::parse).transpose()?.unwrap_or(parsed_config.separator);
        parsed_config.separator_size = style.get("separator_size").map(str::parse).transpose()?.unwrap_or(parsed_config.separator_size);
        parsed_config.value_padding = style.get("value_padding").map(str::parse).transpose()?.unwrap_or(parsed_config.value_padding);
    }

    parsed_config.rendered_separator = parsed_config.separator.repeat(parsed_config.separator_size);

    Ok(parsed_config)
}


fn args_parser(args: Vec<String>) -> Result<Args> {
    let mut parsed_args = Args::default();
    for (i,arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--ss" => {
                let lower_bound = date_str_to_timestamp(args.get(i+1).ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?)?;
                parsed_args.filter = Some(FilterBounds { lower: lower_bound, upper: i64::MAX });
            },
            "--to" => {
                let upper_bound = date_str_to_timestamp(args.get(i+1).ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?)?;
                if let Some(current_filter) = parsed_args.filter {
                    parsed_args.filter = Some(FilterBounds { lower: current_filter.lower, upper: upper_bound });
                } else {
                    parsed_args.filter = Some(FilterBounds { lower: 0, upper: upper_bound })
                }

            },
            "--t" => {
                let current_tp = date_str_to_timestamp(&get_current_date())?;
                let lower_bound = current_tp - (args.get(i+1)
                    .ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?
                    .parse::<i64>().with_context(||"Falha ao ler argumento, consultar --help".bright_red())?
                    *86400 //current date - X in days
                ); 
                parsed_args.filter = Some(FilterBounds { lower: lower_bound, upper: i64::MAX })
            }
            "--help" => {
                parsed_args = Args { filter: None, help: true }
            }
            _ => {}
        }
    }
    Ok(parsed_args)
}

fn add_registry(config: &Config) -> Result<()>{
    let value: f64 = CustomType::new("Valor:")
        .with_formatter(&|i: f64| format!("${}", i))
        .with_error_message("Favor adicionar um número valido")
        .with_help_message("Negativo para gastos, use ponto para centavos")
        .prompt()?;
    
    let desc = Text::new("Descrição:")
        .with_help_message("Descrição para o registro")
        .prompt()?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&config.history_file_path)?;

    let line = format!("{}:{}:{}\n",get_current_date(),value,desc); 
    file.write(line.as_bytes()).expect("Falha ao escrever arquivo.");
    println!("{} Registro adicionado !",">".bright_green());
    Ok(())
}

//Reads the history file and returns a iterator, wihtout consuming it
fn read_history_file(path: &PathBuf) -> Result<impl Iterator<Item = Result<Registry>>>{
    let file = File::open(&path).with_context(||format!("Arquivo de log não existente !\nVocê deve adicionar pelo menos um registro para criar o aquivo.\n> {}",path.to_string_lossy()).bright_red())?;

    let reader = BufReader::new(file);

    Ok(
        reader.lines().map(|line|{
            let line = line?;
            Ok(Registry::from_str(&line).with_context(||"Falha ao parse para registry")?)
        }
    ))
}

fn calculate_and_print_registry(config: &Config, filter: &Option<FilterBounds>) -> Result<()>{
    //Isso some os valores e cria a "janela" de print salvando na memoria só os útlimos X valores para printar
    let mut sum = 0.0;
    let mut print_buf: VecDeque<Registry> = VecDeque::with_capacity(config.trim_size);

    //Lê arquivo faz as somas e filtros
    let mut trim_hide = 0;
    for reg in read_history_file(&config.history_file_path)? {
        let reg = reg?;
        if print_buf.len() == config.trim_size {
            print_buf.pop_front();
        }
        if let Some(filter) = filter {
            let tp = date_str_to_timestamp(&reg.date)?;
            if tp < filter.lower || tp > filter.upper {
                continue;
            }
        };

        sum += reg.money;
        trim_hide += 1;

        print_buf.push_back(reg);
    }

    //Print valores
    if trim_hide > config.trim_size {
        println!("↑\n| Ocultando {} registros",trim_hide-config.trim_size);
    }
    for reg in print_buf {
        let print_money = if reg.money > 0.0 {
            format!("↑${:.2}",reg.money).bright_green()
        } else {
            format!("↓${:.2}",reg.money*-1.0).bright_red()
        };
        let padding = config.value_padding;
        println!("[{}] {:<padding$} {}",reg.date,print_money,reg.desc);
    };

    //Print objetivos
    if let Some(objectives) = &config.objectives {
        println!("{}",&config.rendered_separator);
        for objective in objectives {
            let perc = (sum/objective.1)*100.0;
            let mut perc_str = format!("{:.2}%",perc);
            if perc > 100.0 {
               perc_str = perc_str.bright_green().underline().blink().to_string();
            } else {
                perc_str = perc_str.bright_yellow().to_string();
            }
            println!("{} {}: ${} ({})","❱".bright_green(),objective.0,objective.1,perc_str)
        }
    }

    //Print despesas
    if let Some(expenses) = &config.expenses {
        if !config.objectives.is_some() {
            println!("{}",&config.rendered_separator);
        }
        for expense in expenses {
            let perc = (expense.1/sum)*100.0; //No caso das despesas ela mostra o quantos % da despesa equivale do total E.G: despesa R$100 de Total R$300 = 33.33%
            let mut perc_str = format!("{:.2}%",perc);
            if perc < 20.0 {
               perc_str = perc_str.bright_green().to_string();
            } else if perc < 50.0 {
                perc_str = perc_str.bright_yellow().to_string();
            }else if perc < 80.0 {
                perc_str = perc_str.truecolor(255, 127, 0).to_string(); //laranja
            } else {
                perc_str = perc_str.bright_red().underline().blink().to_string();
            }
            println!("{} {}: ${} ({})","❰".bright_red(),expense.0,expense.1,perc_str)
        }
    }

    println!("{}",&config.rendered_separator);
    println!("Total: ${}",format!("{:.2}",sum).to_string().bright_cyan());
    Ok(())
}

fn main() -> Result<()>{
    let config = config_parser(CONFIG_FILE_NAME.into())?;

    let args = args_parser(env::args().collect::<Vec<String>>())?;

    if args.help {
        println!("{}",include_str!("..\\help.txt"));
        return Ok(())
    }

    if args.filter.is_some() {
        calculate_and_print_registry(&config,&args.filter)?;
    } else {
        let opts = vec!["Ver registros","Adicionar registro"];

        let sel = Select::new("",opts.to_owned())
            .with_help_message("↑↓ para mover")
            .prompt()?;

        if sel == opts[0] {
            calculate_and_print_registry(&config,&args.filter)?;
        } else if sel == opts[1] {
            add_registry(&config)?
        }
    }
    Ok(())
}
