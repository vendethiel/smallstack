#![feature(convert)]
#![feature(slice_patterns)]
#![feature(collections)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Clone)]
enum Expr {
  Int(i64),
  Str(String)
}

struct VM<'a> {
  stack: Vec<Expr>,
  instructions: Vec<&'a str>,
  locals: HashMap<&'a str, Expr>,
}

impl<'a> VM<'a> {
  pub fn new(instructions: Vec<&'a str>) -> VM {
    VM {
      stack: Vec::new(),
      instructions: instructions,
      locals: HashMap::new(),
    }
  }

  fn unsafe_pop(&mut self) -> Expr {
    match self.stack.pop() {
      Some(expr) => expr,
      None => panic!("VM error: stack is empty"),
    }
  }

  pub fn run(&mut self) {
    // just some aliases for brevity
    let len = self.instructions.len();

    // VM variables (might be moved back to the struct itself)
    let mut ip = 0; // instruction pointer
    let mut carry = false;

    while ip < len {
      let instr = self.instructions[ip];
      ip += 1;
      match instr.split(" ").collect::<Vec<_>>().as_slice() {
        ["push", "int", n] => self.stack.push(Expr::Int(n.parse::<i64>().unwrap())),
        ["push", "str", n] => self.stack.push(Expr::Str(String::from_str(n))),

        ["dup"] => {
          let expr = self.unsafe_pop();
          self.stack.push(expr.clone());
          self.stack.push(expr);
        },

        // convert to string
        ["strconv", "int"] => if let Some(&Expr::Int(arg)) = self.stack.last() {
          self.stack.push(Expr::Str(arg.to_string()));
        } else {
          panic!("VM error: cannot convert int to string");
        },

        // how = "carry" | "always"
        ["jump", how, n] => {
          let new_ip = n.parse::<usize>().unwrap();

          if new_ip < len {
            // only check carry if we are conditionally jumping
            if how != "carry" || carry {
              ip = new_ip;
            }
            // only reset carry if it was used
            if how == "carry" {
              carry = false;
            }
          } else {
            panic!("VM error: trying to jump past end of bytecode ({} > {})", new_ip, len);
          }
        },

        ["carry", "set", val] => carry = val == "true",
        ["carry", "invert"] => carry = !carry,

        ["call", "say"] => if let Some(Expr::Str(arg)) = self.stack.pop() {
          println!("hey {}", arg);
        } else {
          panic!("VM error: incorrect arguments to `say`");
        },

        // NOTE: it's stack[*-1] OP stack[*-2]
        // which means if have stack=[1, 2]
        // you'll have 2 OP 1
        /// XXX this should JUST be a call! a call to infix:<...>, that is
        ["math", op] => if let
            (Some(Expr::Int(arg1)), Some(Expr::Int(arg2)))
            =
            (self.stack.pop(), self.stack.pop()) {
          let value = match op {
            "+" => arg1 + arg2,
            "-" => arg1 + arg2,
            //"/" => TODO. this should have very specific rules...
            "*" => arg1 * arg2,
            _ => panic!("VM error: unrecognized `op` operator: {}", op),
          };
          self.stack.push(Expr::Int(value));
        } else {
          panic!("VM error: not enough arguments to `add`");
        },

        // same note about "2 OP 1"
        // XXX same note about how this should be a `call`
        ["cmp", op] => if let
            (Some(Expr::Int(arg1)), Some(Expr::Int(arg2)))
            =
            (self.stack.pop(), self.stack.pop()) {
          carry = match op {
            "<" => arg1 < arg2,
            ">" => arg1 > arg2,
            "<=" => arg1 <= arg2,
            ">=" => arg1 >= arg2,
            "=" => arg1 == arg2,
            _ => panic!("VM error: unrecognized `cmp` operator: {}", op),
          };
        } else {
          panic!("VM error: not enough arguments to `cmp`");
        },

        // TODO this *shouldn't* be an identifier
        //      the backend should transform SSAF to tons of load/store
        //      (probably)
        //      (see backend#1)
        ["local", "load", name] => match self.locals.get(name) {
          Some(expr) => self.stack.push(expr.clone()),
          None       => panic!("VM error: unknown local variable {}", name),
        },

        ["local", "store", name] => if let Some(arg) = self.stack.pop() {
          let _ = self.locals.insert(name, arg);
        } else {
          panic!("VM error: not enough arguments for `local store`");
        },

        [instr, ..] => panic!("VM error: no such instruction {}", instr),
        [..] => panic!("VM error: Unrecognized instruction!"),
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
          // remove trailing newlines, split by line
          let instructions = content.trim_matches('\n').split('\n').collect();
          let mut vm = VM::new(instructions);
          vm.run();
        }
      },
      Err(err) => {
        panic!("Can't read file {} (error: {})", path, err);
      }
    }
	} else {
    panic!("Please give a file argument");
  }

  //     let path = Path::new("chry.fa");
  // let mut file = BufferedReader::new(File::open(&path));
  // for line in file.lines() {
  //  print!("{}", line.unwrap());
  //      }
}
