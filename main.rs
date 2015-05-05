use std::env;
use std::fs::File;
use std::io::Read;

struct VM<'a> {
  stack: Vec<String>,
  instructions: Vec<&'a str>
}

impl<'a> VM<'a> {
  pub fn new(instructions: Vec<&'a str>) -> VM {
    VM {
      stack: Vec::new(),
      instructions: instructions
    }
  }
}

fn main() {
	if let Some(path) = env::args().nth(1) {
		println!("Loading file {}", path);

    match File::open(path.to_string()) {
      Ok(file) => {
        let mut file = file;
        let mut content = String::new();
        if let Ok(_) = file.read_to_string(&mut content) {
          println!("The file contains {}", content);

          let vm = VM::new(content.split('\n').collect());
        }
      },
      Err(err) => {
        println!("Can't read file {} (error: {})", path, err);
      }
    }
	} else {
    println!("Please give a file argument");
  }

  //     let path = Path::new("chry.fa");
  // let mut file = BufferedReader::new(File::open(&path));
  // for line in file.lines() {
  //  print!("{}", line.unwrap());
  //      }
}
