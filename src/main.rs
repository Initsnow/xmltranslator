extern crate quick_xml;

use quick_xml::Reader;
use quick_xml::Writer;
use quick_xml::events::{Event, BytesText};
use std::io::Cursor;

use rust_translate::translate_from_english;

use std::io::{self, Write};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File path to OmenMon.en_US.xml(Dowonload here:https://github.com/OmenMon/Localization/tree/master) template
    #[arg(short, long)]
    path: String,
    /// Translate into what language
    #[arg(short, long)]
    target_language: String,
    /// Jump with tag's attribute key value(I don't know how to jump to the specified exact position, which would jump to the next position(；′⌒`) )
    #[arg(short, long)]
    jump_to_value: Option<String>,
}

#[tokio::main]
async fn main() {
    //Parse env
    let args = Args::parse();
    let Args{path,target_language,jump_to_value}=args;
    // 创建 XML 解析器
    let mut reader = Reader::from_file(path).unwrap();
    reader.trim_text(true);

    // 创建 XML 写入器
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut in_string_tag = false;
    let mut jump_flag = jump_to_value.is_some();
    let mut buf = Vec::new();
    // 解析 XML 并生成新的 XML
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"String" => {
                in_string_tag = true;
                let attrvalue=String::from_utf8(e.try_get_attribute("Key").unwrap().unwrap().value.into_owned()).unwrap();
                if let Some(ref v) = jump_to_value {
                    //println!("attr:{}\tv:{}\t{}",&attrvalue,v,&attrvalue==v);
                    if &attrvalue==v {jump_flag=false;}
                }
            }
            Ok(Event::Start(e)) => {
                // 写入开始标签
                writer.write_event(Event::Start(e)).unwrap();
            }
            Ok(Event::Text(e)) if in_string_tag=> {
                // 修改文本内容并写入
                let text = e.unescape().unwrap();
                //jump
                if jump_flag {
                    let bytes_text = BytesText::new(&text);
                    println!("jump text:{}",text);
                    writer.write_event(Event::Text(bytes_text)).unwrap();
                    continue;
                }
                let translated=&translate_from_english(&text, &target_language).await.unwrap();
                println!("Original text:{}\nTranslated text:{}",text,translated);
                loop {
                    
                    println!("sure? (Y or N or S):");
                    
                    io::stdout().flush().expect("Failed to flush stdout");
            
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Failed to read line");
            
                    let input = input.trim().to_lowercase();
            
                    if input == "y" {
                        let bytes_text = BytesText::new(translated);
                        writer.write_event(Event::Text(bytes_text)).unwrap();
                        break;
                    } else if input == "n" {
                        let mut translatedinput=String::new();
                        println!("Input the translated text:");
                        io::stdin().read_line(&mut translatedinput).expect("Failed to read line");
                        let bytes_text = BytesText::new(translatedinput.trim());
                        writer.write_event(Event::Text(bytes_text)).unwrap();
                        break;
                    } else if input == "s" {
                        let bytes_text = BytesText::new(&text);
                        writer.write_event(Event::Text(bytes_text)).unwrap();
                        break;
                    } else {
                        println!("Invalid input. Please enter 'y' or 'n'.");
                    }
                }
                
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"String" => {
                in_string_tag = false;
            }
            Ok(Event::End(e)) => {
                // 写入结束标签
                writer.write_event(Event::End(e)).unwrap();
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // 忽略其他事件
        }
        buf.clear();
    }

    // 获取生成的 XML 字符串
    let modified_xml = String::from_utf8(writer.into_inner().into_inner()).unwrap();
    println!("{:#?}", modified_xml);
}
