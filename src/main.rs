use std::fs::{OpenOptions,File};
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::path::PathBuf;
use std::env;
use std::collections::VecDeque;
use colored::*;
use regex::{Regex,RegexBuilder};
use inquire::{CustomType,Text,Select};
use chrono::{Local,NaiveDateTime};
use anyhow::{Result,anyhow,Context};
use ini::Ini;

const CONFIG_FILE_NAME: &str = ".fexcel.ini";

struct Config {
    history_file_path: PathBuf,
    objectives: Option<Vec<(String,f64)>>,
    expenses: Option<Vec<(String,f64)>>,
    trim_list_size: usize,
    trim_desc_size: usize,
    separator: String,
    separator_size: usize,
    rendered_separator: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            history_file_path: "history.log".into(),
            objectives: None,
            expenses: None,
            trim_list_size: 50,
            trim_desc_size: usize::MAX,
            separator: "━".to_string(),
            separator_size: 50,
            rendered_separator: "━".to_string().repeat(50)
        }
    }
}

struct Args {
    filter: Option<Filter>,
    help: bool,
    highlights: bool,
    in_out: bool
}

impl Default for Args {
    fn default() -> Self {
        Self {
            filter: None,
            help: false,
            highlights: false,
            in_out: false
        }
    }
}

struct Filter {
    lower: i64,
    upper: i64,
    regex: Option<Regex>
}

impl Default for Filter {
    fn default() -> Self {
        Self { 
            lower: i64::MIN, 
            upper: i64::MAX, 
            regex: None
        }
    }
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
    Local::now().format("%d/%m/%Y").to_string()
}

fn date_str_to_timestamp(date:&str) -> Result<i64> {
    Ok(NaiveDateTime::parse_from_str(&format!("{} 00:00:00",date), "%d/%m/%Y %H:%M:%S").with_context(||anyhow!("Erro ao processar data: {}",date))?.and_utc().timestamp())
}

fn config_parser(config_file_path: PathBuf) -> Result<Config> {
    //If configo file doesn't exist, write one
    if !config_file_path.try_exists()? {
        println!("{}","Sem arquivo de configuração !".bright_yellow());
        println!("{}","Gerando arquivo padrão: .fexcel.ini".bright_yellow());
        let mut file = File::create(&config_file_path)?; 
        file.write_all(include_bytes!("../fexcel_default.ini"))?;
    }

    let mut parsed_config = Config::default();
    let config_file = Ini::load_from_file(&config_file_path)?;

    //pega o caminho do arquivo de history
    parsed_config.history_file_path = config_file.section(None::<String>)
        .ok_or_else(||anyhow!("Falha ao pegar history_file na configuração ! Use: --help !"))?
        .get("history_file_path")
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
            .collect::<Result<Vec<(String,f64)>>>()?; //Rust magia negra, transforma um iter de Result<(k,v),Error> em um iterator de (k,v) e propaga o erro

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
                .collect::<Result<Vec<(String,f64)>>>()?; //Rust magia negra, transforma um iter de Result<(k,v),Error> em um iterator de (k,v) e propaga o erro

        parsed_config.expenses = (!expenses.is_empty()).then_some(expenses)
    }

    if let Some(style) = config_file.section(Some("style")) {
        parsed_config.trim_list_size = style.get("trim_list_size").map(str::parse).transpose()?.unwrap_or(parsed_config.trim_list_size);
        parsed_config.trim_desc_size = style.get("trim_desc_size").map(str::parse).transpose()?.unwrap_or(parsed_config.trim_desc_size);
        parsed_config.separator = style.get("separator").map(str::parse).transpose()?.unwrap_or(parsed_config.separator);
        parsed_config.separator_size = style.get("separator_size").map(str::parse).transpose()?.unwrap_or(parsed_config.separator_size);
    }

    parsed_config.rendered_separator = parsed_config.separator.repeat(parsed_config.separator_size);

    Ok(parsed_config)
}


fn args_parser(args: Vec<String>) -> Result<Args> {
    let mut parsed_args = Args::default();
    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--ss" => {
                let lower_bound = date_str_to_timestamp(args_iter.next().ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?)?;

                let filter = parsed_args.filter.get_or_insert_default();
                filter.lower = lower_bound;
            },
            "--to" => {
                let upper_bound = date_str_to_timestamp(args_iter.next().ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?)?;

                let filter = parsed_args.filter.get_or_insert_default();
                filter.upper = upper_bound;
            },
            "--t" => {
                let current_tp = date_str_to_timestamp(&get_current_date())?;
                let lower_bound = current_tp - (args_iter.next()
                    .ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?
                    .parse::<i64>().with_context(||"Falha ao ler argumento, consultar --help".bright_red())?
                    *86400 //current date - X in days
                ); 

                let filter = parsed_args.filter.get_or_insert_default();
                filter.lower = lower_bound;
            },
            "--r" => {
                let regex_str = args_iter.next().ok_or_else(||anyhow!("Falha ao ler argumento, consultar --help".bright_red()))?;
                let regex = RegexBuilder::new(regex_str)
                    .case_insensitive(true)
                    .build()
                    .with_context(||"Falha ao compilar Regex !")?;

                let filter = parsed_args.filter.get_or_insert_default();
                filter.regex = Some(regex);
            },
            "--hl" => {
                parsed_args.highlights = true;
            }
            "--io" => {
                parsed_args.in_out = true;
            }
            "--help" => {
                parsed_args.help = true;
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

//Reads the history file and returns a iterator, without consuming it
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

fn calculate_and_print_registry(config: &Config, filter: &Option<Filter>) -> Result<()>{
    //Isso some os valores e cria a "janela" de print salvando na memoria só os úLtimos X valores para printar
    let mut sum = 0.0;
    let mut print_buf: VecDeque<Registry> = VecDeque::new();

    //Trim list size override if filtered
    let trim_list_size = if filter.is_some() {
        usize::MAX
    } else {
        config.trim_list_size
    };

    //Lê arquivo faz as somas e filtros
    let mut trim_hide = 0;
    for reg in read_history_file(&config.history_file_path)? {
        let reg = reg?;
        if let Some(filter) = filter {
            let tp = date_str_to_timestamp(&reg.date)?;
            if tp < filter.lower || tp > filter.upper {
                continue;
            };
            if let Some(regex) = &filter.regex {
                if !regex.is_match(&reg.desc) {
                    continue;
                }
            };
        };

        sum += reg.money;
        trim_hide += 1;

        if print_buf.len() == trim_list_size {
            print_buf.pop_front();
        }
        print_buf.push_back(reg);
    }

    //Caso tenha filtrado tudo fora
    if print_buf.len() < 1 {
        println!("{} {}","❱".bright_red(),"Sem dados !");
        return Ok(());
    }

    //Print valores
    if trim_hide > trim_list_size {
        println!("↑\n| Ocultando {} registros",trim_hide-trim_list_size);
    }

    let padding = print_buf.iter().map(|f|format!("{:.2}",f.money.abs()).len() + 2).max().unwrap_or(0);

    for reg in print_buf {
        let print_money = if reg.money > 0.0 {
            format!("↑${:.2}",reg.money).bright_green()
        } else {
            format!("↓${:.2}",reg.money*-1.0).bright_red()
        };

        //adiciona "..." se a descrição for muito longa, definido por trim_desc_size nas configurações
        let mut desc = reg.desc.chars().take(config.trim_desc_size).collect::<String>();
        if reg.desc.chars().count() > config.trim_desc_size {
            desc.push_str("...");
        }

        println!("[{}] {:<padding$} {}",reg.date,print_money,desc);
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

fn print_monthly_in_out(config: &Config) -> Result<()> {
    let mut current_month_in: f64 = 0.0;
    let mut current_month_out: f64 = 0.0;
    let mut current_month_filter: Option<String> = None;

    struct MonthData {
        month: String,
        money_in: f64,
        money_out: f64,
    }

    let mut data_months: Vec<MonthData> = Vec::new();

    for reg in read_history_file(&config.history_file_path)? {
        let reg = reg?; 

        //Pega o resumo de cada mês, entradas e saídas
        let month_year = Some(reg.date[3..].to_string()); // 01/02/2003 => 02/2003
        if month_year != current_month_filter {
            if let Some(month) = current_month_filter {
                data_months.push(
                    MonthData { 
                        month: month,
                        money_in: current_month_in, 
                        money_out: current_month_out
                    }
                );
            }
            current_month_filter = month_year;
            current_month_in = 0.0;
            current_month_out = 0.0;
        }

        if reg.money > 0.0 {
            current_month_in += reg.money
        } else {
            current_month_out += reg.money
        }
    }

    //Adiciona também os dados dá último mês
    if let Some(month) = current_month_filter {
        data_months.push(
            MonthData { 
                month: month,
                money_in: current_month_in, 
                money_out: current_month_out
            }
        );
    }

    let padding = data_months.iter().flat_map(|f|[f.money_in,f.money_out]).map(|v|format!("{:.2}",v.abs()).len()+2).max().unwrap_or(0);
    println!("{}       {}{}{}{}{}",
        "Mês".bold(),
        "Entradas".bold(),
        " ".repeat(padding.checked_sub(5).unwrap_or(0)),
        "Saídas".bold(),
        " ".repeat(padding.checked_sub(3).unwrap_or(0)),
        "Saldo".bold()
    );
    println!("{}",config.separator.repeat((padding*3)+15));
    for data in data_months {
        let rest = data.money_in+data.money_out;
        let print_rest = if rest > 0.0 {
            format!("↑${:.2}",rest).bright_green()
        } else {
            format!("↓${:.2}",rest*-1.0).bright_red()
        };
        println!("{} │ {:<padding$} │ {:<padding$} │ {:<padding$}",
            data.month,
            format!("↑${:.2}",data.money_in).to_string().bright_green(),
            format!("↓${:.2}",data.money_out*-1.0).to_string().bright_red(),
            print_rest
        );
    }
    Ok(())
}

fn print_highlights(config: &Config) -> Result<()> {
    let mut total_sum: f64 = 0.0;
    let mut highest_amount: f64 = 0.0;

    struct HLData {
        date: String,
        value: f64
    }

    let mut highlights: Vec<HLData> = Vec::new();

    for reg in read_history_file(&config.history_file_path)? {
        let reg = reg?; 
        total_sum += reg.money;

        //Pega os "Highlights", os dias em que eu tive aquela quantidade de dinheiro pela primeira vez
        if total_sum > highest_amount {
            highlights.push(
                HLData { 
                    date: reg.date.to_owned(), 
                    value: total_sum
                }
            );
            highest_amount = total_sum;
        }

    }

    let padding = highlights.iter().map(|f|format!("{:.2}",f.value).len()).max().unwrap_or(0) + 3;
    println!("             {}","Highlights".bright_yellow());
    println!("{}",config.separator.repeat(padding+21));
    for hl in highlights {
        println!("[{}]{}{:^padding$}{}",
            hl.date,
            " ━━ ★".bright_yellow(),
            format!("${:.2}",hl.value).to_string().bright_yellow(),
            "★ ━━".bright_yellow());
    }
    Ok(())
}

fn main() -> Result<()>{
    let config = config_parser(CONFIG_FILE_NAME.into())?;

    let args = args_parser(env::args().collect::<Vec<String>>())?;

    if args.help {
        println!("{}",include_str!("../help.txt"));
        return Ok(())
    }
    
    if args.highlights {
        let _ = print_highlights(&config);
        return Ok(());
    }

    if args.in_out {
        let _ = print_monthly_in_out(&config);
        return Ok(());
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
