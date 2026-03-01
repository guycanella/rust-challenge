use std::fs::File;
use std::io::{Write, BufWriter};

fn main() -> std::io::Result<()> {
    let file = File::create("million_rows.csv")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "type,client,tx,amount")?;

    println!("Generating 1.000.000 transactions. Please wait...");

    let total_rows = 1_000_000;
    
    for tx in 1..=total_rows {
        let client = (tx % 10) + 1; 

        if tx % 100 == 10 && tx > 10 {
            let target_tx = tx - 10; 
            writeln!(writer, "dispute,{},{},", client, target_tx)?;
        } 
        else if tx % 100 == 20 && tx > 20 {
            let target_tx = tx - 20; 
            writeln!(writer, "resolve,{},{},", client, target_tx)?;
        }
        else if tx % 100 == 30 && tx > 30 {
            let target_tx = tx - 30;
            writeln!(writer, "dispute,{},{},", client, target_tx)?;
        }
        else if tx % 100 == 40 && tx > 40 {
            let target_tx = tx - 40;
            writeln!(writer, "chargeback,{},{},", client, target_tx)?;
        }
        else if tx % 2 == 0 {
            writeln!(writer, "withdrawal,{},{},1.25", client, tx)?;
        } else {
            writeln!(writer, "deposit,{},{},10.50", client, tx)?;
        }
    }

    writer.flush()?;
    Ok(())
}