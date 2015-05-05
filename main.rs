#![feature(slice_patterns)]
use std::env;
use std::fs::File;
use std::io::Read;

struct VM<'a> {
  stack: Vec<i64>,
  instructions: Vec<&'a str>
}

impl<'a> VM<'a> {
  pub fn new(instructions: Vec<&'a str>) -> VM {
    VM {
      stack: Vec::new(),
      instructions: instructions
    }
  }

  pub fn run(&self) {
    // TODO pass in default stack arguments?
    for instr in self.instructions.iter() {
      match instr.split(" ") {
        ["push", n] => self.stack.push(try!(n.parse::<i64>())),
        ["add"] => {
          if let Some(arg1) = self.stack.pop() && Some(arg2) = self.stack.pop() {
            println!("got {}", arg1 + arg2);
          } else {
            println!("VM error: not enough arguments to `add`");
          }
        },
        [..] => println!("VM error: Unrecognized instruction!") // todo err!
      }
    }
  }
}

fn main() {
	if let Some(path) = env::args().nth(1) {
		println!("Loading file {}", path);

    match File::open(&path) {
      Ok(file) => {
        let mut file = file;
        let mut content = String::new();
        if let Ok(_) = file.read_to_string(&mut content) {
          let vm = VM::new(content.split('\n').collect());
          vm.run();
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
