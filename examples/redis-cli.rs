use clap::Parser;
use clap_repl::ClapEditor;
use redis::Commands;

#[derive(Debug, Parser)]
#[command(name = "")]
enum RedisCommand {
    Get { key: String },
    Set { key: String, value: String },
    Smembers { key: String },
    Sadd { key: String, values: Vec<String> },
}

fn main() {
    let mut client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let rl = ClapEditor::<RedisCommand>::new();
    rl.repl(|command| {
        if let Err(e) = process_command(command, &mut client) {
            println!("{e:?}");
        }
    });
}

fn process_command(command: RedisCommand, client: &mut redis::Client) -> redis::RedisResult<()> {
    match command {
        RedisCommand::Get { key } => {
            let r = client.get::<_, String>(key)?;
            println!("{r}");
        }
        RedisCommand::Set { key, value } => {
            client.set(key, value)?;
            println!("OK");
        }
        RedisCommand::Smembers { key } => {
            let r = client.smembers::<_, Vec<String>>(key)?;
            println!("{r:?}");
        }
        RedisCommand::Sadd { key, values } => {
            let r = client.sadd::<_, _, usize>(key, values)?;
            println!("{r}");
        }
    }
    Ok(())
}
