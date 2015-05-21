#![feature(convert)]
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

  pub fn run(&mut self) {
    // just some aliases for brevity
    let ref mut stack = self.stack;
    let len = self.instructions.len();

    // VM variables (might be moved back to the struct itself)
    let mut ip = 0; // instruction pointer
    let mut carry = false;

    while ip < len {
      let instr = self.instructions[ip];
      ip += 1;
      match instr.split(" ").collect::<Vec<_>>().as_slice() {
        ["push", n] => stack.push(n.parse::<i64>().unwrap()),
        ["add"] => if let (Some(arg1), Some(arg2)) = (stack.pop(), stack.pop()) {
          stack.push(arg1 + arg2);
        } else {
          println!("VM error: not enough arguments to `add`");
        },

        ["say"] => if let Some(arg) = stack.pop() {
          println!("hey {}", arg);
        } else {
          println!("VM error: not enough arguments to `say`");
        },

        ["jump", how, n] => {
          let new_ip = n.parse::<usize>().unwrap();

          if new_ip < len {
            // only check carry if we're not conditionally jumping
            if how != "carry" || carry {
              ip = new_ip;
            }
          } else {
            panic!("VM error: trying to jump past end of bytecode ({} > {})", new_ip, len);
          }
        },

        ["invert_carry"] => carry = !carry,

        [""] => (), // just trap this for now
        [..] => println!("VM error: Unrecognized instruction!"), // todo err!
      }
    }
  }
}

fn main() {
	if let Some(path) = env::args().nth(1) {
		println!("Loading file {}", path);

    match File::open(&path) {
      Ok(mut file) => {
        let mut content = String::new();
        if let Ok(_) = file.read_to_string(&mut content) {
          let mut vm = VM::new(content.split('\n').collect());
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
